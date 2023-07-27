use my_service_bus_tcp_client::MyServiceBusSettings;

use serde_derive::{Deserialize, Serialize};

#[derive(my_settings_reader::SettingsModel, Serialize, Deserialize, Debug, Clone)]
pub struct SettingsModel {
    #[serde(rename = "ServiceBusTcp")]
    pub service_bus_tcp: String,
    #[serde(rename = "DefaultAccountBalance")]
    pub default_account_balance: f64,
    #[serde(rename = "DefaultAccountTradingGroup")]
    pub default_account_trading_group: String,
    #[serde(rename = "AccountsPersistenceGrpcUrl")]
    pub accounts_persistence_grpc_url: String,
    #[serde(rename = "AccountDefaultCurrency")]
    pub accounts_default_currency: Option<String>,
}

impl SettingsReader {
    pub async fn get_default_account_balance_and_group(&self) -> (f64, String) {
        let read_access = self.settings.read().await;
        return (
            read_access.default_account_balance,
            read_access.default_account_trading_group.to_string(),
        );
    }

    pub async fn get_accounts_default_currency(&self) -> Option<String> {
        let read_access = self.settings.read().await;
        return read_access.accounts_default_currency.clone();
    }
}

#[async_trait::async_trait]
impl MyServiceBusSettings for SettingsReader {
    async fn get_host_port(&self) -> String {
        let read_access = self.settings.read().await;
        return read_access.service_bus_tcp.clone();
    }
}

#[async_trait::async_trait]
impl my_grpc_extensions::GrpcClientSettings for SettingsReader {
    async fn get_grpc_url(&self, name: &'static str) -> String {
        if name == crate::grpc_client::AccountsManagerPersistenceGrpcClient::get_service_name() {
            let read_access = self.settings.read().await;
            return read_access.accounts_persistence_grpc_url.clone();
        }

        panic!("Unknown grpc service name: {}", name)
    }
}
