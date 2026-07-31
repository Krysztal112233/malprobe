use malprobe_common::error::Error;
use malprobe_vo::ScanTask;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseBackend, EntityTrait, Set, Statement, Value,
};
use uuid::Uuid;

use crate::helper::SafeTransactionConnectionTrait;
use crate::model::{files, prelude::Files, sea_orm_active_enums::FileStatus};

/// pgmq queue scan tasks are enqueued into (matches the worker default).
pub const SCAN_QUEUE: &str = "scan";

#[async_trait::async_trait]
pub trait FileHelper {
    /// Insert a pending `files` row and enqueue its scan task atomically.
    ///
    /// The `files` row and the pgmq message are committed in one transaction,
    /// so a failure cannot leave an orphaned pending row without a scan task.
    async fn create_pending_file(
        id: impl Into<Uuid> + Send,
        source_url: impl Into<String> + Send,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<files::Model, Error> {
        let id = id.into();
        let tx = database.begin().await?;

        let model = files::ActiveModel {
            id: Set(id),
            sha256: Set(None),
            size: Set(None),
            mime_type: Set(None),
            source_url: Set(source_url.into()),
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
                Value::String(Some(Box::new(SCAN_QUEUE.to_owned()))),
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
}

impl FileHelper for Files {}
