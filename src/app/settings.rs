use my_service_bus_tcp_client::MyServiceBusSettings;
use my_settings_reader::SettingsModel;
use serde_derive::{Deserialize, Serialize};

#[derive(SettingsModel, Serialize, Deserialize, Debug, Clone)]
pub struct SettingsModel {
    #[serde(rename = "ServiceBusTcp")]
    pub service_bus_tcp: String,
    #[serde(rename = "DefaultAccountBalance")]
    pub default_account_balance: f64,
    #[serde(rename = "DefaultAccountTradingGroup")]
    pub default_account_trading_group: String,
    #[serde(rename = "AccountsPersistenceGrpcUrl")]
    pub accounts_persistence_grpc_url: String,
}

#[async_trait::async_trait]
impl MyServiceBusSettings for SettingsModel {
    async fn get_host_port(&self) -> String {
        self.service_bus_tcp.clone()
    }
}
