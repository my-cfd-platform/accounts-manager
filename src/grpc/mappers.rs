use cfd_engine_sb_contracts::AccountBalanceUpdateOperationType;

use crate::accounts_manager::UpdateBalanceReason;

impl Into<AccountBalanceUpdateOperationType> for UpdateBalanceReason {
    fn into(self) -> AccountBalanceUpdateOperationType {
        match self{
            UpdateBalanceReason::TradingResult => AccountBalanceUpdateOperationType::Trading,
            UpdateBalanceReason::BalanceCorrection => AccountBalanceUpdateOperationType::BalanceCorrection,
            UpdateBalanceReason::Deposit => AccountBalanceUpdateOperationType::Deposit,
            UpdateBalanceReason::Withdrawal => AccountBalanceUpdateOperationType::Withdrawal,
            UpdateBalanceReason::WithdrawalCanceled => AccountBalanceUpdateOperationType::Withdrawal,
        }
    }
}