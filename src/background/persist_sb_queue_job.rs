use std::sync::Arc;

use rust_extensions::MyTimerTick;

use crate::AppContext;

pub struct PersistSbQueueJob {
    app: Arc<AppContext>,
}

impl PersistSbQueueJob {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for PersistSbQueueJob {
    async fn tick(&self) {
        self.app.accounts_persist_queue.force_persist().await;
        print!("Done persist job.");
    }
}
