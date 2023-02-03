use serde::{Deserialize, Serialize};

use crate::Account;

#[derive(Serialize, Deserialize, Debug)]
pub enum PersistAccountQueueItem {
    CreateAccount(Account),
    UpdateAccount(Account),
}
