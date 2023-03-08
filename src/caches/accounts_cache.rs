use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::Account;

#[derive(Debug)]
pub enum OperationError {
    TraderNotFound,
    AccountNofFound,
    NotEnoughBalance,
}

impl OperationError {
    pub fn as_grpc_error(&self) -> i32 {
        match self {
            OperationError::TraderNotFound => 2,
            OperationError::AccountNofFound => 1,
            OperationError::NotEnoughBalance => 3,
        }
    }
}

pub struct AccountsStore {
    pub accounts: HashMap<String, HashMap<String, Account>>,
}

impl AccountsStore {
    pub fn new(accounts: Vec<Account>) -> Self {
        let mut accounts_cache = HashMap::new();

        for account in accounts {
            accounts_cache
                .entry(account.trader_id.clone())
                .or_insert(HashMap::new())
                .insert(account.id.clone(), account);
        }

        Self {
            accounts: HashMap::new(),
        }
    }

    pub fn get_account(&self, trader_id: &str, accounts_id: &str) -> Option<&Account> {
        let trader_accounts = self.accounts.get(trader_id)?;
        return trader_accounts.get(accounts_id);
    }

    pub fn get_accounts(&self, trader_id: &str) -> Option<Vec<&Account>> {
        let trader_accounts = self.accounts.get(trader_id)?;
        return Some(trader_accounts.values().collect());
    }

    pub fn add_account(&mut self, account: Account) -> Account {
        let trader_accounts = self
            .accounts
            .entry(account.trader_id.clone())
            .or_insert(HashMap::new());
        trader_accounts.insert(account.id.clone(), account.clone());

        return account;
    }

    pub fn update_balace(
        &mut self,
        trader_id: &str,
        account_id: &str,
        delta: f64,
        process_id: &str,
        allow_negative_balance: bool,
    ) -> Result<&Account, OperationError> {
        let trader_accounts = self.accounts.get_mut(trader_id);

        if let None = trader_accounts {
            return Err(OperationError::TraderNotFound);
        }

        let account = trader_accounts.unwrap().get_mut(account_id);

        if let None = account {
            return Err(OperationError::AccountNofFound);
        }

        let account = account.unwrap();

        if !allow_negative_balance && account.balance + delta < 0.0 {
            return Err(OperationError::NotEnoughBalance);
        }

        account.balance += delta;
        account.last_update_date = chrono::offset::Utc::now().timestamp_millis() as u64;
        account.last_update_process_id = process_id.to_string();

        return Ok(account);
    }

    pub fn update_trading_disabled(
        &mut self,
        trader_id: &str,
        account_id: &str,
        trading_disabled: bool,
        process_id: &str,
    ) -> Result<&Account, OperationError> {
        let trader_accounts = self.accounts.get_mut(trader_id);

        if let None = trader_accounts {
            return Err(OperationError::TraderNotFound);
        }

        let account = trader_accounts.unwrap().get_mut(account_id);

        if let None = account {
            return Err(OperationError::AccountNofFound);
        }

        let account = account.unwrap();

        account.trading_disabled = trading_disabled;
        account.last_update_date = chrono::offset::Utc::now().timestamp_millis() as u64;
        account.last_update_process_id = process_id.to_string();

        return Ok(account);
    }
}

pub struct AccountsCache {
    pub accounts_store: RwLock<AccountsStore>,
}

impl AccountsCache {
    pub fn new(accounts: Vec<Account>) -> Self {
        AccountsCache {
            accounts_store: RwLock::new(AccountsStore::new(accounts)),
        }
    }

    pub async fn get_account(&self, trader_id: &str, accounts_id: &str) -> Option<Account> {
        let accounts_store = self.accounts_store.read().await;
        let account = accounts_store.get_account(trader_id, accounts_id)?.clone();

        return Some(account);
    }

    pub async fn get_accounts(&self, trader_id: &str) -> Option<Vec<Account>> {
        let accounts_store = self.accounts_store.read().await;
        let accounts = accounts_store.get_accounts(trader_id)?;

        let mut result = vec![];

        for itm in accounts {
            result.push(itm.clone());
        }

        return Some(result);
    }

    pub async fn add_account(&self, account: Account) -> Account {
        let mut accounts_store = self.accounts_store.write().await;
        return accounts_store.add_account(account);
    }

    pub async fn update_balance(
        &self,
        trader_id: &str,
        account_id: &str,
        delta: f64,
        process_id: &str,
        allow_negative_balance: bool,
    ) -> Result<Account, OperationError> {
        let mut accounts_store = self.accounts_store.write().await;
        let account = accounts_store.update_balace(
            trader_id,
            account_id,
            delta,
            process_id,
            allow_negative_balance,
        )?;

        return Ok(account.clone());
    }

    pub async fn update_trading_disabled(
        &self,
        trader_id: &str,
        account_id: &str,
        trading_disabled: bool,
        process_id: &str,
    ) -> Result<Account, OperationError> {
        let mut accounts_store = self.accounts_store.write().await;
        let account = accounts_store.update_trading_disabled(
            trader_id,
            account_id,
            trading_disabled,
            process_id,
        )?;

        return Ok(account.clone());
    }
}

// #[cfg(test)]
// mod tests {
//     use stopwatch::Stopwatch;

//     use super::*;

//     #[tokio::test]
//     async fn test_register_cases() {
//         let mut cache = AccountsCache::new();

//         let mut sw_insert = Stopwatch::new();
//         let mut sw_update_balance = Stopwatch::new();
//         let mut sw_trading_disabled = Stopwatch::new();

//         let mut accounts = vec![];

//         for i in 0..2000000 {
//             let guid = uuid::Uuid::new_v4().to_string();

//             let account = Account {
//                 id: guid.to_string(),
//                 trader_id: "1".to_string(),
//                 balance: 100.0,
//                 trading_disabled: false,
//                 last_update_date: 0,
//                 last_update_process_id: "".to_string(),
//                 create_date: 0,
//                 currency: "USD".to_string(),
//                 trading_group: "test".to_string(),
//             };
//             accounts.push(account.clone());
//         }

//         for account in &accounts {
//             let to_insert = account.clone();
//             sw_insert.start();
//             cache.add_account(to_insert).await;
//             sw_insert.stop();
//         }

//         sw_update_balance.start();
//         for account in &accounts {
//             cache
//                 .update_balance(
//                     &account.trader_id,
//                     &account.id,
//                     55.55,
//                     "testtesttest",
//                     true,
//                 )
//                 .await.unwrap();
//         }
//         sw_update_balance.stop();

//         sw_trading_disabled.start();
//         for account in &accounts {
//             cache
//                 .update_trading_disabled(
//                     &account.trader_id,
//                     &account.id,
//                     true,
//                     "testtesttest",
//                 )
//                 .await.unwrap();
//         }
//         sw_trading_disabled.stop();

//         println!(
//             "Add account insert elapsed time: {} ns",
//             sw_insert.elapsed().as_nanos()
//         );
//         println!(
//             "Avg account insert time: {} ns",
//             sw_insert.elapsed().as_nanos() / accounts.len() as u128
//         );
//         println!("------------------------------------------");

//         println!(
//             "Add account update balance elapsed time: {} ns",
//             sw_update_balance.elapsed().as_nanos()
//         );
//         println!(
//             "Avg account update balance time: {} ns",
//             sw_update_balance.elapsed().as_nanos() / accounts.len() as u128
//         );

//         println!("------------------------------------------");
//         println!(
//             "Add account update trading disabled elapsed time: {} ns",
//             sw_update_balance.elapsed().as_nanos()
//         );
//         println!(
//             "Avg account update trading disabled time: {} ns",
//             sw_update_balance.elapsed().as_nanos() / accounts.len() as u128
//         );
//     }
// }
