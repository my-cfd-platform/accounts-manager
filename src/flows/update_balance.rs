use cfd_engine_sb_contracts::{
    AccountBalanceUpdateOperationSbModel, AccountBalanceUpdateOperationType, AccountBalanceUpdateSbModel, AccountPersistEvent
};
use service_sdk::my_telemetry::MyTelemetryContext;

use crate::{
    accounts_manager::AccountManagerUpdateAccountBalanceGrpcRequest, Account, AppContext,
    OperationError,
};

pub async fn update_balance(
    app: &AppContext,
    update_balance_request: &AccountManagerUpdateAccountBalanceGrpcRequest,
    transaction_id: String,
    my_telemetry: &MyTelemetryContext
) -> Result<Account, OperationError> {
    let operation_type: AccountBalanceUpdateOperationType = update_balance_request.reason().into();

    let account_after_update = app
        .accounts_cache
        .update_balance(
            &update_balance_request.trader_id,
            &update_balance_request.account_id,
            update_balance_request.delta,
            &update_balance_request.process_id,
            update_balance_request.allow_negative_balance,
        )
        .await?;

    let balance_update_sb_operation = AccountBalanceUpdateOperationSbModel {
        id: transaction_id.to_string(),
        trader_id: update_balance_request.trader_id.clone(),
        account_id: update_balance_request.account_id.clone(),
        operation_type: operation_type as i32,
        process_id: Some(update_balance_request.process_id.clone()),
        delta: update_balance_request.delta,
        date_time_unix_ms: chrono::offset::Utc::now().timestamp_millis() as u64,
        comment: Some(update_balance_request.comment.clone()),
        reference_operation_id: update_balance_request.reference_transaction_id.clone(),
    };

    let sb_event = AccountPersistEvent{
        add_account_event: None,
        update_account_event:Some(AccountBalanceUpdateSbModel{
            account_after_update: Some(account_after_update.clone().into()),
            operation: Some(balance_update_sb_operation),
        }),
    };

    app.account_persist_events_publisher
        .publish_with_headers(
            &sb_event,
            vec![(
                "type".to_string(),
                app.settings_reader.get_env_type().await,
            )]
            .into(),
            Some(my_telemetry),
        )
        .await
        .unwrap();


    trade_log::trade_log!(
        &update_balance_request.trader_id,
        &update_balance_request.account_id,
        &update_balance_request.process_id,
        &transaction_id,
        "Success update balance operation.",
        my_telemetry.clone(),
        "request" = &update_balance_request,
        "sb_event" = &sb_event
    );


    return Ok(account_after_update);
}
