use engine_sb_contracts::AccountSbModel;
use serde::{Deserialize, Serialize};

use crate::accounts_manager::AccountGrpcModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub currency: String,
    pub trader_id: String,
    pub create_date: u64,
    pub last_update_date: u64,
    pub last_update_process_id: String,
    pub balance: f64,
    pub trading_disabled: bool,
    pub create_process_id: String,
    pub trading_group: String,
}

impl Into<AccountGrpcModel> for Account {
    fn into(self) -> AccountGrpcModel {
        AccountGrpcModel {
            id: self.id,
            trader_id: self.trader_id,
            currency: self.currency,
            balance: self.balance,
            create_date: self.create_date,
            last_update_date: self.last_update_date,
            trading_disabled: self.trading_disabled,
            create_process_id: self.create_process_id,
            trading_group: self.trading_group,
            last_update_process_id: self.last_update_process_id,
        }
    }
}

impl Into<AccountSbModel> for Account {
    fn into(self) -> AccountSbModel {
        AccountSbModel {
            id: self.id,
            trader_id: self.trader_id,
            currency: self.currency,
            balance: self.balance,
            create_date: self.create_date,
            last_update_date: self.last_update_date,
            trading_disabled: self.trading_disabled,
            create_process_id: self.create_process_id,
            trading_group: self.trading_group,
            last_update_process_id: self.last_update_process_id,
        }
    }
}
