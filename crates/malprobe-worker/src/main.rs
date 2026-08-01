use std::future::Future;
use std::path::PathBuf;
use std::time::Duration;

use malprobe_config::{WorkerConfig, WorkerSection};
use malprobe_database::{
    helper::files::FileHelper,
    model::{
        prelude::Files,
        sea_orm_active_enums::{FileStatus, FileVerdict},
    },
};
use malprobe_vo::ScanTask;
use pgmq::{Message, errors::PgmqError, pg_ext::PGMQueueExt};
use sea_orm::DatabaseConnection;
use sha2::Digest;
use tokio::sync::watch;
use tracing::{error, info, warn};
use uuid::Uuid;

mod clamav;

const QUEUE_CREATE_RETRY_INTERVAL: Duration = Duration::from_secs(2);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let config = WorkerConfig::load()?;

    let database = malprobe_database::setup::connect(&config.database).await?;

    let pgmq = PGMQueueExt::new(
        config.database.dsn.clone(),
        config.database.max_connections.unwrap_or(4),
    )
    .await?;

    create_queue_with_retry(&pgmq, &config.worker.queue_name).await?;
    create_queue_with_retry(&pgmq, &config.worker.download_queue_name).await?;
    create_queue_with_retry(&pgmq, &config.worker.enrich_queue_name).await?;

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let mut handles = Vec::with_capacity(
        (config.worker.concurrency
            + config.worker.download_concurrency
            + config.worker.enrich_concurrency) as usize,
    );

    // Scan workers: consume the scan queue and run the actual ClamAV scan.
    for index in 0..config.worker.concurrency {
        let pgmq = pgmq.clone();
        let cfg = config.worker.clone();
        let database = database.clone();
        let queue_name = cfg.queue_name.clone();
        let storage_root = cfg.storage_root.clone();
        let clamd_addr = cfg.clamd_addr.clone();
        let clamd_timeout = Duration::from_secs(cfg.clamd_timeout_seconds);
        let max_read_ct = cfg.max_read_ct;
        let mut shutdown = shutdown_rx.clone();
        handles.push(tokio::spawn(async move {
            worker_loop(
                index,
                pgmq,
                queue_name,
                &cfg,
                move |file_id, read_ct, _msg_id| {
                    let database = database.clone();
                    let storage_root = storage_root.clone();
                    let clamd_addr = clamd_addr.clone();
                    async move {
                        process_scan(
                            &database,
                            &storage_root,
                            &clamd_addr,
                            clamd_timeout,
                            max_read_ct,
                            file_id,
                            read_ct,
                        )
                        .await
                    }
                },
                &mut shutdown,
            )
            .await;
        }));
    }

    // Download workers: consume the download queue, fetch the file bytes,
    // persist them and fan out to the enrich and scan queues.
    for index in 0..config.worker.download_concurrency {
        let pgmq = pgmq.clone();
        let cfg = config.worker.clone();
        let database = database.clone();
        let queue_name = cfg.download_queue_name.clone();
        let storage_root = cfg.storage_root.clone();
        let enrich_queue = cfg.enrich_queue_name.clone();
        let scan_queue = cfg.queue_name.clone();
        let max_read_ct = cfg.max_read_ct;
        let mut shutdown = shutdown_rx.clone();
        handles.push(tokio::spawn(async move {
            worker_loop(
                index,
                pgmq.clone(),
                queue_name.clone(),
                &cfg,
                move |file_id, read_ct, msg_id| {
                    let database = database.clone();
                    let pgmq = pgmq.clone();
                    let queue_name = queue_name.clone();
                    let storage_root = storage_root.clone();
                    let enrich_queue = enrich_queue.clone();
                    let scan_queue = scan_queue.clone();
                    async move {
                        let ctx = StageCtx {
                            database: &database,
                            pgmq: &pgmq,
                            queue_name: &queue_name,
                            storage_root: &storage_root,
                            max_read_ct,
                        };
                        process_download(&ctx, &enrich_queue, &scan_queue, file_id, read_ct, msg_id)
                            .await
                    }
                },
                &mut shutdown,
            )
            .await;
        }));
    }

    // Enrich workers: consume the enrich queue, sniff the file type from the
    // stored bytes and backfill the metadata.
    for index in 0..config.worker.enrich_concurrency {
        let pgmq = pgmq.clone();
        let cfg = config.worker.clone();
        let database = database.clone();
        let queue_name = cfg.enrich_queue_name.clone();
        let storage_root = cfg.storage_root.clone();
        let max_read_ct = cfg.max_read_ct;
        let mut shutdown = shutdown_rx.clone();
        handles.push(tokio::spawn(async move {
            worker_loop(
                index,
                pgmq.clone(),
                queue_name.clone(),
                &cfg,
                move |file_id, read_ct, msg_id| {
                    let database = database.clone();
                    let pgmq = pgmq.clone();
                    let queue_name = queue_name.clone();
                    let storage_root = storage_root.clone();
                    async move {
                        let ctx = StageCtx {
                            database: &database,
                            pgmq: &pgmq,
                            queue_name: &queue_name,
                            storage_root: &storage_root,
                            max_read_ct,
                        };
                        process_enrich(&ctx, file_id, read_ct, msg_id).await
                    }
                },
                &mut shutdown,
            )
            .await;
        }));
    }

    tokio::signal::ctrl_c().await?;
    info!("shutdown signal received, waiting for workers to stop");
    let _ = shutdown_tx.send(true);
    for handle in handles {
        let _ = handle.await;
    }
    info!("all workers stopped");

    Ok(())
}

/// Polls one queue forever. One `worker_loop` per configured concurrency slot.
///
/// `process` gets the file id, the queue read count and the message id;
/// anything else it needs (the pgmq client, the database, the queue to
/// enqueue into) is captured by the closure.
async fn worker_loop<F, Fut>(
    index: u32,
    pgmq: PGMQueueExt,
    queue_name: String,
    cfg: &WorkerSection,
    process: F,
    shutdown: &mut watch::Receiver<bool>,
) where
    F: Fn(Uuid, u32, i64) -> Fut + Send + Sync,
    Fut: Future<Output = Result<(), String>> + Send,
{
    let vt = Duration::from_secs(cfg.vt_seconds);
    let poll_timeout = Duration::from_millis(cfg.poll_timeout_ms);
    let poll_interval = Duration::from_millis(cfg.poll_interval_ms);

    loop {
        tokio::select! {
            _ = shutdown.changed() => {
                info!(worker = index, queue = %queue_name, "shutdown signal received");
                break;
            }
            result = pgmq.read_with_poll::<ScanTask>(
                &queue_name,
                vt,
                Some(poll_timeout),
                Some(poll_interval),
            ) => {
                match result {
                    Ok(Some(message)) => {
                        process_message(&pgmq, &queue_name, &message, &process).await
                    }
                    Ok(None) => {}
                    Err(error) => {
                        error!(worker = index, queue = %queue_name, %error, "failed to read from queue");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }
    }
}

/// Handles a single queue message.
///
/// On success the message is deleted. On failure the message is left in the
/// queue: it becomes visible again after the visibility timeout, which gives
/// us retry for free. The process closure receives `read_ct` (so a stage can
/// give up after a configured number of attempts and archive the message)
/// and the message id (needed by `pgmq.archive`).
async fn process_message<F, Fut>(
    pgmq: &PGMQueueExt,
    queue_name: &str,
    message: &Message<ScanTask>,
    process: &F,
) where
    F: Fn(Uuid, u32, i64) -> Fut + Send + Sync,
    Fut: Future<Output = Result<(), String>> + Send,
{
    let file_id = message.message.file_id;
    info!(msg_id = message.msg_id, read_ct = message.read_ct, %file_id, queue = %queue_name, "received task");

    match process(file_id, message.read_ct.max(0) as u32, message.msg_id).await {
        Ok(()) => {
            if let Err(error) = pgmq.delete(queue_name, message.msg_id).await {
                error!(msg_id = message.msg_id, %error, "failed to delete message from queue");
            }
        }
        Err(error) => {
            warn!(msg_id = message.msg_id, %error, "failed to process message, it will be retried after the visibility timeout");
        }
    }
}

/// Scan stage: run a ClamAV INSTREAM scan over the stored bytes and persist
/// the verdict.
///
/// Transient failures (clamd unreachable, scan error) leave the message in
/// the queue so the visibility timeout retries it; after `max_read_ct` read
/// attempts the scan is marked as permanently failed and the message is
/// dropped. The stored bytes are removed only once the verdict is persisted,
/// so a retry always finds its input.
async fn process_scan(
    database: &DatabaseConnection,
    storage_root: &str,
    clamd_addr: &str,
    clamd_timeout: Duration,
    max_read_ct: u32,
    file_id: Uuid,
    read_ct: u32,
) -> Result<(), String> {
    // Idempotency guard: a retried message (pgmq delete failed, crash before
    // delete, duplicate fan-out) may arrive after the file already reached a
    // terminal state. Skipping here keeps a duplicate from overwriting the
    // verdict with `scanning` and eventually `failed`.
    match Files::find_by_id(file_id, database)
        .await
        .map_err(|e| format!("failed to look up file {file_id}: {e}"))?
    {
        None => {
            warn!(%file_id, "file row not found, dropping scan task");
            return Ok(());
        }
        Some(model) if matches!(model.status, FileStatus::Completed | FileStatus::Failed) => {
            info!(%file_id, status = ?model.status, "file already in terminal state, treating retry as done");
            return Ok(());
        }
        Some(_) => {}
    }

    let path = PathBuf::from(storage_root).join(file_id.to_string());
    if let Err(error) = Files::mark_scanning(file_id, database).await {
        return fail_or_retry(
            database,
            &path,
            file_id,
            read_ct,
            max_read_ct,
            &format!("failed to mark file as scanning: {error}"),
        )
        .await;
    }

    let outcome = match clamav::scan_path(&path, clamd_addr, clamd_timeout).await {
        Ok(clamav::ScanVerdict::Clean) => {
            info!(%file_id, "scan finished, file is clean");
            Files::mark_completed(file_id, Some(FileVerdict::Clean), None, database).await
        }
        Ok(clamav::ScanVerdict::Found(signature)) => {
            warn!(%file_id, %signature, "malware detected");
            Files::mark_completed(
                file_id,
                Some(FileVerdict::Malicious),
                Some(signature),
                database,
            )
            .await
        }
        Err(error) => {
            return fail_or_retry(database, &path, file_id, read_ct, max_read_ct, &error).await;
        }
    };

    // The verdict is only persisted once, after the scan itself succeeded; a
    // persistence error is a scan-stage failure like any other.
    if let Err(error) = outcome {
        return fail_or_retry(
            database,
            &path,
            file_id,
            read_ct,
            max_read_ct,
            &format!("failed to record scan verdict: {error}"),
        )
        .await;
    }

    // Only remove the bytes once the verdict is persisted: a failed scan
    // must keep its input for the next retry.
    if let Err(error) = tokio::fs::remove_file(&path).await {
        warn!(%file_id, %error, "failed to remove scanned file bytes");
    }

    Ok(())
}

/// Applies the give-up policy after any scan-stage failure.
///
/// While `read_ct` is below `max_read_ct` the error is returned so the queue
/// visibility timeout retries the message. Once the limit is reached:
///
/// - the file is marked failed via a conditional update that skips completed
///   rows, so a concurrent successful scan never loses its verdict;
/// - only a successful marking removes the bytes and drops the message;
/// - a failed marking (database trouble) returns an error instead, keeping
///   both the message and the bytes so the retry can finalize once the
///   database recovers.
async fn fail_or_retry(
    database: &DatabaseConnection,
    path: &std::path::Path,
    file_id: Uuid,
    read_ct: u32,
    max_read_ct: u32,
    error: &str,
) -> Result<(), String> {
    if !should_give_up(read_ct, max_read_ct) {
        return Err(error.to_owned());
    }

    warn!(%file_id, read_ct, max_read_ct, %error, "scan failed permanently, marking file as failed");
    match Files::mark_failed(file_id, error, database).await {
        Ok(true) => {
            // The row was ours to finalize; no retry needs the bytes anymore.
            if let Err(e) = tokio::fs::remove_file(path).await {
                warn!(%file_id, %e, "failed to remove failed scan bytes");
            }
            Ok(())
        }
        Ok(false) => {
            // Already completed by a concurrent scan (or the row is gone):
            // the standing verdict wins, drop the stale message.
            info!(%file_id, "file already finalized by another attempt, dropping message");
            Ok(())
        }
        Err(e) => {
            // Keep the message and the bytes: retrying is self-healing once
            // the database is reachable again, dropping would strand the row
            // in `scanning` forever.
            Err(format!("failed to record scan failure: {e}"))
        }
    }
}

/// Whether the give-up policy applies for a message read `read_ct` times
/// with a configured limit of `max_read_ct` attempts.
///
/// A message read at least `max_read_ct` times is abandoned instead of
/// retried; `max_read_ct = 0` means no retry is allowed at all.
fn should_give_up(read_ct: u32, max_read_ct: u32) -> bool {
    read_ct >= max_read_ct
}

/// Shared per-stage context: the dependencies every queue stage needs.
struct StageCtx<'a> {
    database: &'a DatabaseConnection,
    pgmq: &'a PGMQueueExt,
    /// The queue this stage consumes; also the archive target on give-up.
    queue_name: &'a str,
    storage_root: &'a str,
    max_read_ct: u32,
}

/// Download stage: fetch the file bytes from `source_url`, persist them under
/// `{storage_root}/{file_id}` and fan out to the enrich and scan queues.
///
/// Transient failures are retried via the queue visibility timeout; after
/// `max_read_ct` attempts the file is marked failed and the message is
/// archived. Permanently unprocessable tasks (missing row/source) are
/// archived immediately.
async fn process_download(
    ctx: &StageCtx<'_>,
    enrich_queue: &str,
    scan_queue: &str,
    file_id: Uuid,
    read_ct: u32,
    msg_id: i64,
) -> Result<(), String> {
    let Some(model) = Files::find_by_id(file_id, ctx.database)
        .await
        .map_err(|e| format!("failed to look up file {file_id}: {e}"))?
    else {
        warn!(%file_id, "file row not found, archiving download task");
        return archive_message(ctx, msg_id, file_id).await;
    };

    let source = match model.source {
        Some(source) => source,
        None => {
            warn!(%file_id, "file has no download source, archiving download task");
            return archive_message(ctx, msg_id, file_id).await;
        }
    };
    // Treat non-2xx responses as download failures (a 404 error page is not
    // the file bytes), and route every failure through the give-up policy so
    // a permanently broken source cannot retry forever.
    let response = match reqwest::get(&source).await {
        Ok(response) => response,
        Err(e) => {
            return download_fail_or_archive(
                ctx,
                file_id,
                read_ct,
                msg_id,
                &format!("failed to download {source}: {e}"),
            )
            .await;
        }
    };
    let status = response.status();
    if !status.is_success() {
        return download_fail_or_archive(
            ctx,
            file_id,
            read_ct,
            msg_id,
            &format!("failed to download {source}: HTTP {status}"),
        )
        .await;
    }
    let bytes = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(e) => {
            return download_fail_or_archive(
                ctx,
                file_id,
                read_ct,
                msg_id,
                &format!("failed to read response body: {e}"),
            )
            .await;
        }
    };

    // Persist the bytes before acknowledging anything: the enrich and scan
    // stages read the file from disk by file id. Write to a temp path and
    // rename so a crash never leaves a half-written file behind.
    let dir = PathBuf::from(ctx.storage_root);
    let path = dir.join(file_id.to_string());
    if let Err(error) = tokio::fs::create_dir_all(&dir).await {
        return download_fail_or_archive(
            ctx,
            file_id,
            read_ct,
            msg_id,
            &format!("failed to create storage dir: {error}"),
        )
        .await;
    }
    let tmp = path.with_extension("tmp");
    if let Err(error) = tokio::fs::write(&tmp, &bytes).await {
        return download_fail_or_archive(
            ctx,
            file_id,
            read_ct,
            msg_id,
            &format!("failed to write {}: {error}", tmp.display()),
        )
        .await;
    }
    if let Err(error) = tokio::fs::rename(&tmp, &path).await {
        return download_fail_or_archive(
            ctx,
            file_id,
            read_ct,
            msg_id,
            &format!("failed to persist {}: {error}", path.display()),
        )
        .await;
    }

    let sha256 = hex::encode(sha2::Sha256::digest(&bytes));
    if let Err(error) =
        Files::mark_downloaded(file_id, sha256, bytes.len() as i64, ctx.database).await
    {
        return download_fail_or_archive(
            ctx,
            file_id,
            read_ct,
            msg_id,
            &format!("failed to backfill metadata: {error}"),
        )
        .await;
    }

    // Enrich and scan run in parallel: both only read the persisted bytes and
    // fail independently. Once the bytes are persisted and metadata is
    // backfilled the download stage is done; a fan-out failure is retried as
    // a whole (the idempotent write would just re-run).
    if let Err(error) = pgmq_send(ctx, enrich_queue, file_id).await {
        return download_fail_or_archive(
            ctx,
            file_id,
            read_ct,
            msg_id,
            &format!("failed to enqueue enrich task: {error}"),
        )
        .await;
    }
    if let Err(error) = pgmq_send(ctx, scan_queue, file_id).await {
        return download_fail_or_archive(
            ctx,
            file_id,
            read_ct,
            msg_id,
            &format!("failed to enqueue scan task: {error}"),
        )
        .await;
    }

    info!(%file_id, size = bytes.len(), "file downloaded, enrich and scan tasks enqueued");

    Ok(())
}

/// Sends a scan task into `queue` via the stage's pgmq client.
async fn pgmq_send(ctx: &StageCtx<'_>, queue: &str, file_id: Uuid) -> Result<(), String> {
    ctx.pgmq
        .send(queue, &ScanTask { file_id })
        .await
        .map_err(|e| format!("failed to enqueue task: {e}"))?;
    Ok(())
}

/// Download failure handler: retry until `max_read_ct`, then mark the file
/// permanently failed and archive the message.
async fn download_fail_or_archive(
    ctx: &StageCtx<'_>,
    file_id: Uuid,
    read_ct: u32,
    msg_id: i64,
    error: &str,
) -> Result<(), String> {
    if !should_give_up(read_ct, ctx.max_read_ct) {
        return Err(error.to_owned());
    }

    warn!(%file_id, read_ct, max_read_ct = ctx.max_read_ct, %error, "download failed permanently, marking file as failed");
    if let Err(e) = Files::mark_failed(file_id, error, ctx.database).await {
        warn!(%file_id, %e, "failed to record download failure");
    }
    archive_message(ctx, msg_id, file_id).await
}

/// Moves a message out of its queue into the pgmq archive table.
async fn archive_message(ctx: &StageCtx<'_>, msg_id: i64, file_id: Uuid) -> Result<(), String> {
    ctx.pgmq
        .archive(ctx.queue_name, msg_id)
        .await
        .map_err(|e| format!("failed to archive message: {e}"))?;
    info!(%file_id, queue = %ctx.queue_name, "message archived");
    Ok(())
}

/// Enrich stage: sniff the file type from the stored bytes and backfill the
/// mime type. Runs in parallel with the scan stage and never touches the
/// status, so a sniff failure (or an unknown type) does not block scanning.
///
/// A missing file with the row still in a non-terminal state is retried
/// until `max_read_ct`, then the message is archived: enrichment is
/// best-effort and a missing mime type is acceptable (it must not mark the
/// file failed — that terminal state belongs to the scan stage).
async fn process_enrich(
    ctx: &StageCtx<'_>,
    file_id: Uuid,
    read_ct: u32,
    msg_id: i64,
) -> Result<(), String> {
    let path = PathBuf::from(ctx.storage_root).join(file_id.to_string());
    let bytes = match tokio::fs::read(&path).await {
        Ok(bytes) => bytes,
        Err(error) if !path.exists() => {
            // The scan stage removes the bytes once its verdict is persisted;
            // if the file is already gone the scan reached a terminal state
            // (completed or permanently failed) and there is nothing left to
            // sniff.
            let model = Files::find_by_id(file_id, ctx.database)
                .await
                .map_err(|e| format!("failed to look up file {file_id}: {e}"))?;
            if model.is_some_and(|m| matches!(m.status, FileStatus::Completed | FileStatus::Failed))
            {
                info!(%file_id, "file scan already finished, skipping enrichment");
                return Ok(());
            }
            if should_give_up(read_ct, ctx.max_read_ct) {
                warn!(%file_id, read_ct, max_read_ct = ctx.max_read_ct, "file bytes missing and never finalized, archiving enrich task");
                return archive_message(ctx, msg_id, file_id).await;
            }
            return Err(format!("failed to read {}: {error}", path.display()));
        }
        Err(error) => return Err(format!("failed to read {}: {error}", path.display())),
    };

    match infer::get(&bytes) {
        Some(info) => {
            Files::mark_enriched(file_id, info.mime_type(), ctx.database)
                .await
                .map_err(|e| format!("failed to backfill mime type: {e}"))?;
            info!(%file_id, mime = info.mime_type(), "file enriched");
        }
        None => {
            info!(%file_id, "file type not recognized, skipping enrichment");
        }
    }

    Ok(())
}

/// `pgmq.create` is idempotent; retry until it succeeds so that the worker can
/// start before the backend ran the migration that creates the pgmq extension.
async fn create_queue_with_retry(pgmq: &PGMQueueExt, queue_name: &str) -> Result<(), PgmqError> {
    loop {
        match pgmq.create(queue_name).await {
            Ok(_) => {
                info!(queue = queue_name, "queue ready");
                return Ok(());
            }
            Err(error) => {
                warn!(queue = queue_name, %error, "failed to create queue, retrying (waiting for the pgmq extension?)");
                tokio::time::sleep(QUEUE_CREATE_RETRY_INTERVAL).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gives_up_at_the_limit() {
        // Exactly `max_read_ct` reads are tolerated, the next one gives up.
        assert!(!should_give_up(4, 5));
        assert!(should_give_up(5, 5));
        assert!(should_give_up(6, 5));
    }

    #[test]
    fn gives_up_on_first_failure_with_limit_one() {
        assert!(!should_give_up(0, 1));
        assert!(should_give_up(1, 1));
    }

    #[test]
    fn zero_limit_never_retries() {
        // A limit of 0 disables retries entirely: any failed attempt gives up.
        assert!(should_give_up(0, 0));
        assert!(should_give_up(1, 0));
    }
}
