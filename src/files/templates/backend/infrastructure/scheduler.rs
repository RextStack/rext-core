use apalis::prelude::*;
use apalis_cron::{CronStream, Schedule};
use apalis_sql::sqlite::SqliteStorage;
use chrono::{DateTime, Utc};
use sea_orm::sqlx::SqlitePool;
use serde::{Deserialize, Serialize};
use std::{io::Error, str::FromStr};

/// Reminder structure for scheduled tasks
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Reminder(DateTime<Utc>);

impl From<DateTime<Utc>> for Reminder {
    fn from(t: DateTime<Utc>) -> Self {
        Reminder(t)
    }
}

/// Task scheduler manager
pub struct SchedulerManager;

impl SchedulerManager {
    /// Handles a tick job
    pub async fn handle_tick(job: Reminder) -> Result<(), Error> {
        println!("Handling tick: {:?}", job);
        Ok(())
    }

    /// Creates and runs the task scheduler
    pub async fn run_scheduler(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Create DB pool for cron
        let cron_pool = SqlitePool::connect(database_url).await.unwrap();
        let schedule = Schedule::from_str("0 */1 * * * *").unwrap(); // every minute
        println!("Starting cron worker with schedule: {}", schedule);

        let cron_stream = CronStream::new(schedule);
        let sqlite_storage = SqliteStorage::new(cron_pool);
        let cron_backend = cron_stream.pipe_to_storage(sqlite_storage);

        let worker = WorkerBuilder::new("morning-cereal")
            .backend(cron_backend)
            .build_fn(Self::handle_tick);

        Monitor::new().register(worker).run().await.unwrap();
        Ok(())
    }
}
