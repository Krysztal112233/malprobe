use std::time::Duration;

use malprobe_config::{WorkerConfig, WorkerSection};
use pgmq::{Message, errors::PgmqError, pg_ext::PGMQueueExt};
use tokio::sync::watch;
use tracing::{error, info, warn};
use uuid::Uuid;

mod task;

use task::ScanTask;

const QUEUE_CREATE_RETRY_INTERVAL: Duration = Duration::from_secs(2);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let config = WorkerConfig::load()?;

    let pgmq = PGMQueueExt::new(
        config.database.dsn.clone(),
        config.database.max_connections.unwrap_or(4),
    )
    .await?;

    create_queue_with_retry(&pgmq, &config.worker.queue_name).await?;

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let mut handles = Vec::with_capacity(config.worker.concurrency as usize);
    for index in 0..config.worker.concurrency {
        let pgmq = pgmq.clone();
        let cfg = config.worker.clone();
        let mut shutdown = shutdown_rx.clone();
        handles.push(tokio::spawn(async move {
            worker_loop(index, pgmq, cfg, &mut shutdown).await;
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

/// Polls the queue forever. One `worker_loop` per configured concurrency slot.
async fn worker_loop(
    index: u32,
    pgmq: PGMQueueExt,
    cfg: WorkerSection,
    shutdown: &mut watch::Receiver<bool>,
) {
    let vt = Duration::from_secs(cfg.vt_seconds);
    let poll_timeout = Duration::from_millis(cfg.poll_timeout_ms);
    let poll_interval = Duration::from_millis(cfg.poll_interval_ms);

    loop {
        tokio::select! {
            _ = shutdown.changed() => {
                info!(worker = index, "shutdown signal received");
                break;
            }
            result = pgmq.read_with_poll::<ScanTask>(
                &cfg.queue_name,
                vt,
                Some(poll_timeout),
                Some(poll_interval),
            ) => {
                match result {
                    Ok(Some(message)) => process_message(&pgmq, &cfg.queue_name, &message).await,
                    Ok(None) => {}
                    Err(error) => {
                        error!(worker = index, %error, "failed to read from queue");
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
async fn process_message(pgmq: &PGMQueueExt, queue_name: &str, message: &Message<ScanTask>) {
    let file_id = message.message.file_id;
    info!(msg_id = message.msg_id, read_ct = message.read_ct, %file_id, "received scan task");

    match process(&file_id).await {
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

/// Skeleton for the actual scan: look up the `files` row, transition to
/// `scanning`, run the ClamAV INSTREAM scan over the stored file, persist the
/// verdict. Not implemented yet.
async fn process(file_id: &Uuid) -> Result<(), String> {
    warn!(%file_id, "scan not implemented yet, treating message as processed");
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
