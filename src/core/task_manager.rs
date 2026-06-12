use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};
use tokio::task::JoinHandle;

#[derive(Clone, Debug)]
pub enum RestartPolicy { Never, OnFailure { max_retries: u32 }, Always }

pub struct TaskHandle { pub join: JoinHandle<()>, pub stop_tx: oneshot::Sender<()> }

pub struct TaskManager { inner: Arc<Mutex<HashMap<String, TaskHandle>>> }

impl TaskManager {
    pub fn new() -> Self { Self { inner: Arc::new(Mutex::new(HashMap::new())) } }

    pub async fn spawn_named<Fut, F>(&self, name: &str, f: F, _policy: RestartPolicy) -> Result<()>
    where F: FnOnce() -> Fut + Send + 'static, Fut: std::future::Future<Output=()> + Send + 'static {
        let (tx, rx) = oneshot::channel::<()>();
        let name_s = name.to_string();
        let join = tokio::spawn(async move {
            tokio::select! {
                _ = f() => {}
                _ = rx => {}
            }
        });
        let mut m = self.inner.lock().await;
        m.insert(name_s, TaskHandle { join, stop_tx: tx });
        Ok(())
    }

    /// Spawn a periodic blocking task using spawn_blocking. The provided closure `f` will be executed in a blocking thread.
    pub async fn spawn_periodic_blocking<F>(&self, name: &str, interval: std::time::Duration, f: F) -> Result<()>
    where
        F: Fn() + Send + Sync + 'static,
    {
        let f = std::sync::Arc::new(f);
        let name_s = name.to_string();
        let (tx, rx) = oneshot::channel::<()>();

        let join = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            // Run once immediately
            let f0 = f.clone();
            let _ = tokio::task::spawn_blocking(move || {
                (f0)();
            })
            .await;

            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        let f_cl = f.clone();
                        let _ = tokio::task::spawn_blocking(move || {
                            (f_cl)();
                        }).await;
                    }
                    _ = &mut rx => {
                        break;
                    }
                }
            }
        });

        let mut m = self.inner.lock().await;
        m.insert(name_s, TaskHandle { join, stop_tx: tx });
        Ok(())
    }

    pub async fn stop(&self, name: &str) -> Result<()> {
        let mut m = self.inner.lock().await;
        if let Some(h) = m.remove(name) {
            let _ = h.stop_tx.send(());
        }
        Ok(())
    }

    pub async fn status(&self, name: &str) -> Option<TaskStatus> {
        let m = self.inner.lock().await;
        if let Some(_) = m.get(name) {
            Some(TaskStatus { running: true })
        } else {
            Some(TaskStatus { running: false })
        }
    }
}

pub struct TaskStatus { pub running: bool }
