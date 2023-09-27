use serde::{Deserialize, Serialize};
use service_sdk::async_trait;

service_sdk::macros::use_settings!();

#[derive(
    my_settings_reader::SettingsModel, AutoGenerateSettingsTraits, SdkSettingsTraits, Serialize, Deserialize, Debug, Clone,
)]
pub struct SettingsModel {
    pub my_sb_tcp_host_port: String,
    pub default_account_balance: f64,
    pub default_account_trading_group: String,
    pub accounts_persistence_grpc_url: String,
    pub accounts_default_currency: Option<String>,
    pub my_telemetry: String,
    pub seq_conn_string: String,
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
impl service_sdk::my_grpc_extensions::GrpcClientSettings for SettingsReader {
    async fn get_grpc_url(&self, name: &'static str) -> String {
        if name == crate::grpc_client::AccountsManagerPersistenceGrpcClient::get_service_name() {
            let read_access = self.settings.read().await;
            return read_access.accounts_persistence_grpc_url.clone();
        }

        panic!("Unknown grpc service name: {}", name)
    }
}
