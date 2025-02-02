use std::time::Duration;
use futures::prelude::*;
use futures::stream::FuturesUnordered;
use tokio::time::timeout;

use crate::{ArbitrageResult, Context, SharedRef};

pub type SpawnResult = tokio::task::JoinHandle<ArbitrageResult<String>>;
pub type WorkerRef = Box<dyn Worker + Send + Sync>;

/// A trait that defines an interface for a worker
pub trait Worker {
    /// Spawns a new worker into a tokio task
    fn spawn(&mut self) -> SpawnResult;

    /// Checks if the worker is running
    fn is_running(&self) -> bool {
        true
    }
}

#[derive(Default, Clone)]
pub struct RunningFlag {
    running: SharedRef<bool>,
}

impl RunningFlag {
    pub fn stop(&self) {
        *self.running.lock() = false;
    }

    pub fn is_running(&self) -> bool {
        *self.running.lock()
    }

    pub fn start(&self) {
        *self.running.lock() = true;
    }
}

pub struct Workers {
    context: Context,
    delay_millis: u64,
    workers: Vec<WorkerRef>,
    running: RunningFlag
}


impl Workers {
    pub fn new(context: Context, delay_millis: u64) -> Self {
        Self {
            context,
            delay_millis,
            workers: vec![],
            running: RunningFlag::default(),
        }
    }

    pub fn add_worker(&mut self, worker: WorkerRef) {
        self.workers.push(worker);
    }

    pub async fn run(&mut self) -> ArbitrageResult<String> {
        self.spawn().await.unwrap()
    }
}


impl Clone for Workers {
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
            delay_millis: self.delay_millis,
            workers: vec![],
            running: self.running.clone(),
        }
    }
}


impl Worker for Workers {
    fn spawn(&mut self) -> SpawnResult {
        let workers = self.workers.drain(..).collect::<Vec<WorkerRef>>();
        let running = self.running.clone();
        let context = self.context.clone();
        let delay_millis = self.delay_millis;
        // A timeout indiciating that if the worker does not start within this time, it will be considered failed
        // and the worker will be stopped
        let timeout_millis = context.config.get_int("worker_timeout_millis").unwrap_or(5000) as u64;

        log::info!("Starting workers for {}", context.name);
        tokio::spawn(async move {
            running.start();
            tokio::time::sleep(Duration::from_millis(delay_millis)).await;

            let mut futures = vec![];
            for mut worker in workers {
                futures.push(worker.spawn());
            }

            log::info!("{} spawned {} workers", context.name, futures.len());
            // Convert the vector of futures to a futures unordered as it allows us to
            // run all the futures concurrently and get results as they complete, without
            // enforcing a specific order
            let mut futures = futures.into_iter().collect::<FuturesUnordered<_>>();

            // Wait for the first future to complete
            if let Some(result) = futures.next().await {
                if let Ok(result) = result {
                    match result {
                        Err(err) => {
                            log::error!("{} worker failed with error: {:?}", context.name, err);
                        }
                        Ok(name) => {
                            log::info!("worker {} exited", name);
                        }
                    }
                } else {
                    log::error!("worker error - {:?}", result);
                }
            }

            // Sending exit signal to the other workers
            if !context.exit() {
                log::error!("failed to exit");
            } else {
                // Wait for other workers to complete. The idea is if we consolidate multiple workers
                // and if one worker completed, it means other workers should also be done.
                // We wait for 5 seconds for other workers to complete or we consider them failed
                // and exit the application
                log::warn!("waiting for other workers to complete");
                let timeout_duration = Duration::from_millis(timeout_millis);

                match timeout(timeout_duration, async {
                    while let Some(result) = futures.next().await {
                        if let Ok(result) = result {
                            match result {
                                Err(err) => {
                                    log::error!("{} worker failed with error: {:?}", context.name, err);
                                }
                                Ok(name) => {
                                    log::info!("worker {} exited", name);
                                }
                            }
                        } else {
                            log::error!("worker error - {:?}", result);
                        }
                    }
                }).await {
                    Ok(_) => {
                        log::info!("all workers exited");
                    }
                    Err(_) => {
                        log::error!("{} workers did not exit within timeout of {} ms", context.name, timeout_millis);
                    }
                }


            }
            running.stop();
            context.log_and_exit("stopped")
        })

    }
}
