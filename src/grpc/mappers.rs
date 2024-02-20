use cfd_engine_sb_contracts::AccountBalanceUpdateOperationType;

use crate::{
    accounts_manager::{AccountManagerGetClientAccountGrpcResponse, AccountsManagerOperationResult, UpdateBalanceReason},
    Account,
};

impl Into<AccountBalanceUpdateOperationType> for UpdateBalanceReason {
    fn into(self) -> AccountBalanceUpdateOperationType {
        match self {
            UpdateBalanceReason::TradingResult => AccountBalanceUpdateOperationType::Trading,
            UpdateBalanceReason::BalanceCorrection => {
                AccountBalanceUpdateOperationType::BalanceCorrection
            }
            UpdateBalanceReason::Deposit => AccountBalanceUpdateOperationType::Deposit,
            UpdateBalanceReason::Withdrawal => AccountBalanceUpdateOperationType::Withdrawal,
            UpdateBalanceReason::WithdrawalCanceled => {
                AccountBalanceUpdateOperationType::Withdrawal
            }
        }
    }
}

impl Into<AccountManagerGetClientAccountGrpcResponse> for Option<Account> {
    fn into(self) -> AccountManagerGetClientAccountGrpcResponse {
        match self {
            Some(account) => AccountManagerGetClientAccountGrpcResponse {
                result: AccountsManagerOperationResult::Ok as i32,
                account: Some(account.into()),
            },
            None => AccountManagerGetClientAccountGrpcResponse {
                result: AccountsManagerOperationResult::AccountNotFound as i32,
                account: None,
            },
        }
    }
}

