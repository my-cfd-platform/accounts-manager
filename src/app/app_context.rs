use std::sync::Arc;

use engine_sb_contracts::AccountPersistEvent;
use my_service_bus_abstractions::publisher::MyServiceBusPublisher;
use my_service_bus_tcp_client::MyServiceBusClient;
use persist_queue::{PersistentQueue, PersistentQueueSettings};
use rust_extensions::AppStates;

use crate::{AccountsCache, PersistAccountQueueItem, SettingsModel};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");

pub struct AppContext {
    pub accounts_cache: Arc<AccountsCache>,
    pub accounts_persist_queue: Arc<PersistentQueue<PersistAccountQueueItem>>,
    pub settings: Arc<SettingsModel>,
    pub app_states: Arc<AppStates>,
    pub sb_client: MyServiceBusClient,
    pub account_persist_events_publisher: MyServiceBusPublisher<AccountPersistEvent>,
}

impl AppContext {
    pub async fn new(settings: Arc<SettingsModel>) -> Self {
        let persist_queue = PersistentQueue::load_from_backup(
            "AccountsSbPersistQueue".to_string(),
            PersistentQueueSettings::FilePersist(
                "./backup/accounts-sb-persist-queue".to_string(),
                2,
            ),
        )
        .await;

        let sb_client = MyServiceBusClient::new(
            APP_NAME,
            APP_VERSION,
            settings.clone(),
            my_logger::LOGGER.clone(),
        );

        let account_persist_events_publisher = sb_client.get_publisher(false).await;

        Self {
            accounts_cache: Arc::new(AccountsCache::new()),
            settings,
            app_states: Arc::new(AppStates::create_initialized()),
            accounts_persist_queue: Arc::new(persist_queue),
            sb_client,
            account_persist_events_publisher,
        }
    }
}
