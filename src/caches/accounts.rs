use cfd_engine_sb_contracts::{AccountSbMetadataModel, AccountSbModel};
use serde::{Deserialize, Serialize};

use crate::{
    accounts_manager::{AccountGrpcModel, AccountMetadataItemGrpcModel},
    accounts_manager_persistence::PersistenceAccountGrpcModel,
};

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
    pub metadata: Vec<AccountMetadataItemGrpcModel>,
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
            metadata: self.metadata,
        }
    }
}

impl Into<Account> for PersistenceAccountGrpcModel {
    fn into(self) -> Account {
        Account {
            id: self.id,
            currency: self.currency,
            trader_id: self.trader_id,
            create_date: self.create_date,
            last_update_date: self.last_update_date,
            last_update_process_id: self.last_update_process_id,
            balance: self.balance,
            trading_disabled: self.trading_disabled,
            create_process_id: self.create_process_id,
            trading_group: self.trading_group,
            metadata: self
                .metadata
                .into_iter()
                .map(|x| AccountMetadataItemGrpcModel {
                    key: x.key,
                    value: x.value,
                })
                .collect(),
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
            metadata: self
                .metadata
                .into_iter()
                .map(|x| AccountSbMetadataModel {
                    key: x.key,
                    value: x.value,
                })
                .collect(),
        }
    }
}
