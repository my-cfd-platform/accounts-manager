use my_settings_reader::SettingsModel;
use serde_derive::{Serialize, Deserialize};

#[derive(SettingsModel, Serialize, Deserialize, Debug, Clone)]
pub struct SettingsModel{
    #[serde(rename = "DefaultAccountBalance")]
    pub default_account_balance: f64,
    #[serde(rename = "DefaultAccountTradingGroup")]
    pub default_account_trading_group: String,
}