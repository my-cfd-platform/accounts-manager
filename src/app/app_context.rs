use std::sync::Arc;

use cfd_engine_sb_contracts::AccountPersistEvent;
use service_sdk::my_service_bus::abstractions::publisher::MyServiceBusPublisher;
use service_sdk::my_telemetry::MyTelemetryContext;
use service_sdk::ServiceContext;

use crate::accounts_manager::AccountManagerUpdateAccountBalanceGrpcResponse;
use crate::accounts_manager_persistence::GetAllAccountsGrpcRequest;
use crate::{AccountsCache, ProcessIdCache, SettingsReader};

use crate::grpc_client::AccountsManagerPersistenceGrpcClient;
pub struct AppContext {
    pub accounts_cache: Arc<AccountsCache>,
    pub settings_reader: Arc<SettingsReader>,
    pub account_persist_events_publisher: MyServiceBusPublisher<AccountPersistEvent>,
    pub cache: ProcessIdCache<AccountManagerUpdateAccountBalanceGrpcResponse>
}

impl AppContext {
    pub async fn new(settings_reader: Arc<SettingsReader>, sc: &ServiceContext) -> Self {
        let account_persist_events_publisher = sc.get_sb_publisher(false).await;
        Self {
            accounts_cache: Arc::new(load_accounts(settings_reader.clone()).await),
            settings_reader,
            account_persist_events_publisher,
            cache: ProcessIdCache::new()
        }
    }
}

async fn load_accounts(settings_reader: Arc<SettingsReader>) -> AccountsCache {
    let settings = settings_reader.get_settings().await;
    let accounts_persistence_grpc =
        AccountsManagerPersistenceGrpcClient::new(settings_reader.clone());

    let telemetry = MyTelemetryContext::new();
    telemetry.start_event_tracking("load_accounts");

    let accounts = accounts_persistence_grpc
        .get_all_accounts(GetAllAccountsGrpcRequest{
            accounts_type: settings._type,
        }, &telemetry)
        .await
        .unwrap();

    let accounts = match accounts {
        Some(src) => src,
        None => vec![],
    };

    println!("Load {} accounts from persistence", accounts.len());

    return AccountsCache::new(accounts.iter().map(|x| x.to_owned().into()).collect());
}
