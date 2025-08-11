use apalis::prelude::*;
use apalis_sql::sqlite::SqliteStorage;
use chrono::Utc;
use sea_orm::sqlx::SqlitePool;
use serde::{Deserialize, Serialize};
use std::io::Error;

/// Message structure for job queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub to: String,
    pub text: String,
    pub subject: String,
}

/// Job queue manager
pub struct JobQueueManager;

impl JobQueueManager {
    /// Creates job storage from pool
    pub fn create_storage(pool: SqlitePool) -> SqliteStorage<Message> {
        SqliteStorage::new(pool)
    }

    /// Produces test messages for the job queue
    pub async fn produce_messages(storage: &SqliteStorage<Message>) -> Result<(), Error> {
        let mut storage = storage.clone();
        for i in 0..1 {
            storage
                .schedule(
                    Message {
                        to: format!("test{i}@example.com"),
                        text: "Test background job from apalis".to_string(),
                        subject: "Background email job".to_string(),
                    },
                    (Utc::now() + chrono::Duration::seconds(4)).timestamp(),
                )
                .await
                .unwrap();
        }
        Ok(())
    }

    /// Sends a message (job handler)
    pub async fn send_message(message: Message) -> Result<(), Error> {
        println!("Sending message: {:?}", message);
        Ok(())
    }

    /// Creates and runs the job queue monitor
    pub async fn run_job_queue_monitor(
        job_storage: SqliteStorage<Message>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Monitor::new()
            .register({
                WorkerBuilder::new("tasty-banana")
                    .backend(job_storage)
                    .build_fn(Self::send_message)
            })
            .run()
            .await
            .unwrap();
        Ok(())
    }
}
