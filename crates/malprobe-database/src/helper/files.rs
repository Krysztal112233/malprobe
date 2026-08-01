use malprobe_common::error::Error;
use malprobe_vo::ScanTask;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseBackend, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set, Statement, Value,
};
use uuid::Uuid;

use crate::helper::SafeTransactionConnectionTrait;
use crate::model::{
    files,
    prelude::Files,
    sea_orm_active_enums::{FileSourceType, FileStatus, FileVerdict},
};

/// pgmq queue download tasks are enqueued into by the backend; the worker
/// downloads the file bytes and enqueues a scan task afterwards.
pub const DOWNLOAD_QUEUE: &str = "download";

/// pgmq queue scan tasks are enqueued into by the download worker.
pub const SCAN_QUEUE: &str = "scan";

/// pgmq queue enrichment tasks are enqueued into by the download worker; the
/// enrich stage sniffs the file type and backfills metadata.
pub const ENRICH_QUEUE: &str = "enrich";

#[async_trait::async_trait]
pub trait FileHelper {
    /// Insert a pending `files` row and enqueue its download task atomically.
    ///
    /// The `files` row and the pgmq message are committed in one transaction,
    /// so a failure cannot leave an orphaned pending row without a download task.
    async fn create_pending_file(
        id: impl Into<Uuid> + Send,
        source_type: FileSourceType,
        source: impl Into<String> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<files::Model, Error> {
        let id = id.into();
        let tx = database.begin().await?;

        let model = files::ActiveModel {
            id: Set(id),
            sha256: Set(None),
            size: Set(None),
            mime_type: Set(None),
            source: Set(Some(source.into())),
            source_type: Set(source_type),
            status: Set(FileStatus::Pending),
            verdict: Set(None),
            malware_name: Set(None),
            details: Set(None),
            error: Set(None),
            // created_at/updated_at are filled by the DB defaults.
            created_at: Default::default(),
            updated_at: Default::default(),
            scanned_at: Set(None),
        }
        .insert(&tx)
        .await?;

        tx.execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT pgmq.send($1, $2::jsonb)",
            vec![
                Value::String(Some(Box::new(DOWNLOAD_QUEUE.to_owned()))),
                Value::Json(Some(Box::new(
                    serde_json::to_value(ScanTask { file_id: id }).map_err(|e| {
                        Error::Unknown(format!("failed to serialize scan task: {e}"))
                    })?,
                ))),
            ],
        ))
        .await?;

        tx.commit().await?;

        Ok(model)
    }

    /// Fetch a file row by its primary key.
    async fn find_by_id(
        id: impl Into<Uuid> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Option<files::Model>, Error> {
        Ok(<Files as EntityTrait>::find_by_id(id.into())
            .one(database)
            .await?)
    }

    /// Fetch every scan record for a file hash, newest first.
    ///
    /// The same bytes can be submitted from different sources, each keeping
    /// its own record; deduplication is the caller's job.
    async fn find_by_hash(
        sha256: impl Into<String> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<files::Model>, Error> {
        Ok(files::Entity::find()
            .filter(files::Column::Sha256.eq(sha256.into()))
            .order_by_desc(files::Column::CreatedAt)
            .all(database)
            .await?)
    }

    /// Fetch one page of scan records, newest first, plus the total row count.
    ///
    /// `page` is 1-based; 0 is rejected so a caller mistake fails loudly
    /// instead of underflowing into `fetch_page(u64::MAX)`.
    async fn find_page(
        page: u64,
        size: u64,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(Vec<files::Model>, u64), Error> {
        if page == 0 {
            return Err(Error::Unknown(format!("page must be >= 1, got {page}")));
        }
        let paginator = files::Entity::find()
            .order_by_desc(files::Column::CreatedAt)
            .paginate(database, size);
        let total = paginator.num_items().await?;
        // `fetch_page` is 0-based.
        let models = paginator.fetch_page(page - 1).await?;
        Ok((models, total))
    }

    /// Make sure the scan queue exists. `pgmq.create` is idempotent; the
    /// backend must ensure the queue exists because it can run before any
    /// worker started.
    async fn ensure_scan_queue(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        database
            .execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "SELECT pgmq.create($1)",
                vec![Value::String(Some(Box::new(SCAN_QUEUE.to_owned())))],
            ))
            .await?;
        Ok(())
    }

    /// Make sure the download queue exists. See [`Self::ensure_scan_queue`].
    async fn ensure_download_queue(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        database
            .execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "SELECT pgmq.create($1)",
                vec![Value::String(Some(Box::new(DOWNLOAD_QUEUE.to_owned())))],
            ))
            .await?;
        Ok(())
    }

    /// Make sure the enrich queue exists. See [`Self::ensure_scan_queue`].
    async fn ensure_enrich_queue(
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        database
            .execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "SELECT pgmq.create($1)",
                vec![Value::String(Some(Box::new(ENRICH_QUEUE.to_owned())))],
            ))
            .await?;
        Ok(())
    }

    /// Backfill the metadata the downloader learned about the file bytes.
    /// The status stays `pending`; the scan worker moves it to `scanning`.
    async fn mark_downloaded(
        id: impl Into<Uuid> + Send,
        sha256: impl Into<String> + Send,
        size: impl Into<i64> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        let id = id.into();
        files::ActiveModel {
            id: Set(id),
            sha256: Set(Some(sha256.into())),
            size: Set(Some(size.into())),
            ..Default::default()
        }
        .update(database)
        .await?;
        Ok(())
    }

    /// Backfill the mime type sniffed from the file bytes by the enrich stage.
    async fn mark_enriched(
        id: impl Into<Uuid> + Send,
        mime_type: impl Into<String> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        files::ActiveModel {
            id: Set(id.into()),
            mime_type: Set(Some(mime_type.into())),
            ..Default::default()
        }
        .update(database)
        .await?;
        Ok(())
    }

    /// Move the file into the `scanning` state while the ClamAV scan runs.
    async fn mark_scanning(
        id: impl Into<Uuid> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        files::ActiveModel {
            id: Set(id.into()),
            status: Set(FileStatus::Scanning),
            ..Default::default()
        }
        .update(database)
        .await?;
        Ok(())
    }

    /// Record the terminal outcome of a finished scan (status `completed`).
    async fn mark_completed(
        id: impl Into<Uuid> + Send,
        verdict: Option<FileVerdict>,
        malware_name: Option<String>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        files::ActiveModel {
            id: Set(id.into()),
            status: Set(FileStatus::Completed),
            verdict: Set(verdict),
            malware_name: Set(malware_name),
            scanned_at: Set(Some(chrono::Utc::now().into())),
            ..Default::default()
        }
        .update(database)
        .await?;
        Ok(())
    }

    /// Record a scan that gave up after repeated failures (status `failed`).
    ///
    /// Only rows that are not already completed are touched: a stale retry
    /// racing with a concurrent successful scan must not overwrite the
    /// verdict. Returns whether a row was updated (`false` means the file was
    /// already finalized by someone else, or is gone).
    async fn mark_failed(
        id: impl Into<Uuid> + Send,
        error: impl Into<String> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<bool, Error> {
        let result = files::Entity::update_many()
            .set(files::ActiveModel {
                status: Set(FileStatus::Failed),
                error: Set(Some(error.into())),
                ..Default::default()
            })
            .filter(files::Column::Id.eq(id.into()))
            .filter(files::Column::Status.ne(FileStatus::Completed))
            .exec(database)
            .await?;
        Ok(result.rows_affected > 0)
    }
}

impl FileHelper for Files {}
