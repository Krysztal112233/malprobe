use std::future::Future;
use std::time::Duration;

use malprobe_config::{WorkerConfig, WorkerSection};
use malprobe_database::helper::files::FileHelper;
use malprobe_database::model::prelude::Files;
use malprobe_vo::ScanTask;
use pgmq::{Message, errors::PgmqError, pg_ext::PGMQueueExt};
use reqwest::header::CONTENT_TYPE;
use sea_orm::DatabaseConnection;
use sha2::Digest;
use tokio::sync::watch;
use tracing::{error, info, warn};
use uuid::Uuid;

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

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let mut handles = Vec::with_capacity(
        (config.worker.concurrency + config.worker.download_concurrency) as usize,
    );

    // Scan workers: consume the scan queue and run the actual ClamAV scan.
    for index in 0..config.worker.concurrency {
        let pgmq = pgmq.clone();
        let cfg = config.worker.clone();
        let queue_name = cfg.queue_name.clone();
        let mut shutdown = shutdown_rx.clone();
        handles.push(tokio::spawn(async move {
            worker_loop(index, pgmq, queue_name, &cfg, process_scan, &mut shutdown).await;
        }));
    }

    // Download workers: consume the download queue, fetch the file bytes and
    // enqueue a scan task once the metadata is backfilled.
    for index in 0..config.worker.download_concurrency {
        let pgmq = pgmq.clone();
        let cfg = config.worker.clone();
        let database = database.clone();
        let queue_name = cfg.download_queue_name.clone();
        let scan_queue = cfg.queue_name.clone();
        let mut shutdown = shutdown_rx.clone();
        handles.push(tokio::spawn(async move {
            worker_loop(
                index,
                pgmq.clone(),
                queue_name,
                &cfg,
                move |file_id| {
                    let database = database.clone();
                    let pgmq = pgmq.clone();
                    let scan_queue = scan_queue.clone();
                    async move { process_download(&database, &pgmq, &scan_queue, file_id).await }
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
/// `process` gets only the file id; anything else it needs (the pgmq client,
/// the database, the queue to enqueue into) is captured by the closure.
async fn worker_loop<F, Fut>(
    index: u32,
    pgmq: PGMQueueExt,
    queue_name: String,
    cfg: &WorkerSection,
    process: F,
    shutdown: &mut watch::Receiver<bool>,
) where
    F: Fn(Uuid) -> Fut + Send + Sync,
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
/// us retry for free. (TODO: when `read_ct` reaches a configured maximum,
/// archive the message and mark the scan as permanently failed instead.)
async fn process_message<F, Fut>(
    pgmq: &PGMQueueExt,
    queue_name: &str,
    message: &Message<ScanTask>,
    process: &F,
) where
    F: Fn(Uuid) -> Fut + Send + Sync,
    Fut: Future<Output = Result<(), String>> + Send,
{
    let file_id = message.message.file_id;
    info!(msg_id = message.msg_id, read_ct = message.read_ct, %file_id, queue = %queue_name, "received task");

    match process(file_id).await {
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

/// Scan stage skeleton: run the ClamAV INSTREAM scan over the downloaded
/// bytes and persist the verdict. Not implemented yet.
async fn process_scan(file_id: Uuid) -> Result<(), String> {
    warn!(%file_id, "scan not implemented yet, treating message as processed");
    Ok(())
}

/// Download stage: fetch the file bytes from `source_url`, backfill the
/// metadata and enqueue the scan task.
async fn process_download(
    database: &DatabaseConnection,
    pgmq: &PGMQueueExt,
    scan_queue: &str,
    file_id: Uuid,
) -> Result<(), String> {
    let Some(model) = Files::find_by_id(file_id, database)
        .await
        .map_err(|e| format!("failed to look up file {file_id}: {e}"))?
    else {
        return Err(format!("file {file_id} not found"));
    };

    let response = reqwest::get(&model.source_url)
        .await
        .map_err(|e| format!("failed to download {}: {e}", model.source_url))?;
    let mime_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("failed to read response body: {e}"))?;

    let sha256 = hex::encode(sha2::Sha256::digest(&bytes));
    Files::mark_downloaded(file_id, sha256, bytes.len() as i64, mime_type, database)
        .await
        .map_err(|e| format!("failed to backfill metadata: {e}"))?;

    pgmq.send(scan_queue, &ScanTask { file_id })
        .await
        .map_err(|e| format!("failed to enqueue scan task: {e}"))?;

    info!(%file_id, size = bytes.len(), "file downloaded, scan task enqueued");

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
