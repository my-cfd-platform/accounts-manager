use std::{pin::Pin, vec};

use crate::accounts_manager::{
    AccountManagerGetTraderIdByAccountIdGrpcRequest,
    AccountManagerGetTraderIdByAccountIdGrpcResponse, SearchAccounts, AccountManagerUpdateTradingGroupGrpcRequest,
};
use crate::{
    accounts_manager::{
        accounts_manager_grpc_service_server::AccountsManagerGrpcService, AccountGrpcModel,
        AccountManagerCreateAccountGrpcRequest, AccountManagerGetClientAccountGrpcRequest,
        AccountManagerGetClientAccountGrpcResponse, AccountManagerGetClientAccountsGrpcRequest,
        AccountManagerUpdateAccountBalanceGrpcRequest,
        AccountManagerUpdateAccountBalanceGrpcResponse, AccountManagerUpdateBalanceBalanceGrpcInfo,
        AccountManagerUpdateTradingDisabledGrpcRequest,
        AccountManagerUpdateTradingDisabledGrpcResponse,
    },
    Account,
};
use cfd_engine_sb_contracts::{
    AccountBalanceUpdateOperationSbModel, AccountBalanceUpdateSbModel, AccountPersistEvent,
};
use service_sdk::my_grpc_extensions::prelude::Stream;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use super::server::GrpcService;
use service_sdk::my_grpc_extensions;
use service_sdk::my_grpc_extensions::server::with_telemetry;

#[tonic::async_trait]
impl AccountsManagerGrpcService for GrpcService {
    type GetClientAccountsStream = Pin<
        Box<dyn Stream<Item = Result<AccountGrpcModel, tonic::Status>> + Send + Sync + 'static>,
    >;

    #[with_telemetry]
    async fn create_account(
        &self,
        request: tonic::Request<AccountManagerCreateAccountGrpcRequest>,
    ) -> Result<tonic::Response<AccountGrpcModel>, tonic::Status> {
        let request = request.into_inner();

        let (default_account_balance, default_account_trading_group) = self
            .app
            .settings_reader
            .get_default_account_balance_and_group()
            .await;

        let tg = match request.trading_group_id {
            Some(tg) => tg,
            None => default_account_trading_group,
        };

        let date = chrono::offset::Utc::now().timestamp_millis() as u64;
        let account_to_insert = Account {
            id: Uuid::new_v4().to_string(),
            balance: default_account_balance,
            currency: request.currency,
            trader_id: request.trader_id,
            trading_disabled: false,
            create_date: date,
            last_update_date: date,
            last_update_process_id: request.process_id.clone(),
            create_process_id: request.process_id.clone(),
            trading_group: tg,
        };

        let account = self
            .app
            .accounts_cache
            .add_account(account_to_insert.clone())
            .await;

        self.app
            .account_persist_events_publisher
            .publish(
                &AccountPersistEvent {
                    add_account_event: Some(account.clone().into()),
                    update_account_event: None,
                },
                Some(my_telemetry),
            )
            .await
            .unwrap();

        return Ok(tonic::Response::new(account_to_insert.into()));
    }

    #[with_telemetry]
    async fn get_client_account(
        &self,
        request: tonic::Request<AccountManagerGetClientAccountGrpcRequest>,
    ) -> Result<tonic::Response<AccountManagerGetClientAccountGrpcResponse>, tonic::Status> {
        let request  = request.into_inner();
        
        let AccountManagerGetClientAccountGrpcRequest {
            trader_id,
            account_id,
        } = request;
        let account = self
            .app
            .accounts_cache
            .get_account(&trader_id, &account_id)
            .await;

        let response = match account {
            Some(account) => tonic::Response::new(AccountManagerGetClientAccountGrpcResponse {
                result: 0,
                account: Some(account.into()),
            }),
            None => tonic::Response::new(AccountManagerGetClientAccountGrpcResponse {
                result: 1,
                account: None,
            }),
        };

        Ok(response)
    }

    #[with_telemetry]
    async fn get_client_accounts(
        &self,
        request: tonic::Request<AccountManagerGetClientAccountsGrpcRequest>,
    ) -> Result<tonic::Response<Self::GetClientAccountsStream>, tonic::Status> {
        let request = request.into_inner();
        let AccountManagerGetClientAccountsGrpcRequest { trader_id } = request;
        let accounts = self.app.accounts_cache.get_accounts(&trader_id).await;

        let accounts = match accounts {
            Some(accounts) => accounts
                .iter()
                .map(|x| x.to_owned().into())
                .collect::<Vec<AccountGrpcModel>>(),
            None => match &self
                .app
                .settings_reader
                .get_accounts_default_currency()
                .await
            {
                Some(currency) => {
                    let request = AccountManagerCreateAccountGrpcRequest {
                        trader_id: trader_id.clone(),
                        currency: currency.clone(),
                        process_id: Uuid::new_v4().to_string(),
                        trading_group_id: None,
                    };

                    let account = self
                        .create_account(Request::new(request))
                        .await
                        .unwrap()
                        .into_inner();

                    vec![account.into()]
                }
                None => vec![],
            },
        };

        service_sdk::my_grpc_extensions::grpc_server::send_vec_to_stream(accounts, |x| x).await
    }

    #[with_telemetry]
    async fn update_client_account_balance(
        &self,
        request: tonic::Request<AccountManagerUpdateAccountBalanceGrpcRequest>,
    ) -> Result<tonic::Response<AccountManagerUpdateAccountBalanceGrpcResponse>, tonic::Status>
    {
        let request = request.into_inner();
        let transaction_id = Uuid::new_v4().to_string();

        let balance_operation = AccountBalanceUpdateOperationSbModel {
            id: transaction_id.clone(),
            trader_id: request.trader_id.clone(),
            account_id: request.account_id.clone(),
            operation_type: request.reason.clone(),
            process_id: Some(request.process_id.clone()),
            delta: request.delta,
            date_time_unix_ms: chrono::offset::Utc::now().timestamp_millis() as u64,
            comment: Some(request.comment),
            reference_operation_id: request.reference_transaction_id,
        };

        let update_balance_result = self
            .app
            .accounts_cache
            .update_balance(
                &request.trader_id,
                &request.account_id,
                request.delta,
                &request.process_id,
                request.allow_negative_balance,
            )
            .await;

        let response = match update_balance_result {
            Ok(account) => {
                self.app
                    .account_persist_events_publisher
                    .publish(
                        &AccountPersistEvent {
                            add_account_event: None,
                            // update_account_event: Some(account.clone().into()),
                            update_account_event: Some(AccountBalanceUpdateSbModel {
                                account_after_update: Some(account.clone().into()),
                                operation: Some(balance_operation),
                            }),
                        },
                        Some(my_telemetry),
                    )
                    .await
                    .unwrap();

                AccountManagerUpdateAccountBalanceGrpcResponse {
                    result: 0,
                    update_balance_info: Some(AccountManagerUpdateBalanceBalanceGrpcInfo {
                        account: Some(account.into()),
                        operation_id: transaction_id,
                    }),
                }
            }
            Err(error) => AccountManagerUpdateAccountBalanceGrpcResponse {
                result: error.as_grpc_error(),
                update_balance_info: None,
            },
        };

        Ok(tonic::Response::new(response))
    }

    #[with_telemetry]
    async fn update_account_trading_disabled(
        &self,
        request: tonic::Request<AccountManagerUpdateTradingDisabledGrpcRequest>,
    ) -> Result<tonic::Response<AccountManagerUpdateTradingDisabledGrpcResponse>, tonic::Status>
    {
        let request = request.into_inner();

        let update_balance_result = self
            .app
            .accounts_cache
            .update_trading_disabled(
                &request.trader_id,
                &request.account_id,
                request.trading_disabled,
                &request.process_id,
            )
            .await;

        let response = match update_balance_result {
            Ok(account) => {
                self.app
                    .account_persist_events_publisher
                    .publish(
                        &AccountPersistEvent {
                            add_account_event: None,
                            update_account_event: Some(AccountBalanceUpdateSbModel {
                                account_after_update: Some(account.clone().into()),
                                operation: None,
                            }),
                        },
                        Some(my_telemetry),
                    )
                    .await
                    .unwrap();
                AccountManagerUpdateTradingDisabledGrpcResponse {
                    result: 0,
                    account: Some(account.into()),
                }
            }
            Err(error) => AccountManagerUpdateTradingDisabledGrpcResponse {
                result: error.as_grpc_error(),
                account: None,
            },
        };

        Ok(tonic::Response::new(response))
    }

    #[with_telemetry]
    async fn update_account_trading_group(
        &self,
        request: tonic::Request<AccountManagerUpdateTradingGroupGrpcRequest>,
    ) -> Result<tonic::Response<AccountManagerUpdateTradingDisabledGrpcResponse>, tonic::Status>
    {
        let request = request.into_inner();

        let update_balance_result = self
            .app
            .accounts_cache
            .update_trading_group(
                &request.trader_id,
                &request.account_id,
                request.new_trading_group.as_str(),
                &request.process_id,
            )
            .await;

        let response = match update_balance_result {
            Ok(account) => {
                self.app
                    .account_persist_events_publisher
                    .publish(
                        &AccountPersistEvent {
                            add_account_event: None,
                            update_account_event: Some(AccountBalanceUpdateSbModel {
                                account_after_update: Some(account.clone().into()),
                                operation: None,
                            }),
                        },
                        Some(my_telemetry),
                    )
                    .await
                    .unwrap();
                AccountManagerUpdateTradingDisabledGrpcResponse {
                    result: 0,
                    account: Some(account.into()),
                }
            }
            Err(error) => AccountManagerUpdateTradingDisabledGrpcResponse {
                result: error.as_grpc_error(),
                account: None,
            },
        };

        Ok(tonic::Response::new(response))
    }

    #[with_telemetry]
    async fn get_trader_id_by_account_id(
        &self,
        request: Request<AccountManagerGetTraderIdByAccountIdGrpcRequest>,
    ) -> Result<Response<AccountManagerGetTraderIdByAccountIdGrpcResponse>, Status> {
        let request = request.into_inner();
        let account_id = request.account_id;

        let result = self
            .app
            .accounts_cache
            .get_trader_id_by_account_id(account_id.as_str())
            .await;
        Ok(Response::new(
            AccountManagerGetTraderIdByAccountIdGrpcResponse { trader_id: result },
        ))
    }

    type SearchStream = Pin<
        Box<dyn Stream<Item = Result<AccountGrpcModel, tonic::Status>> + Send + Sync + 'static>,
    >;

    #[with_telemetry]
    async fn search(
        &self,
        request: Request<SearchAccounts>,
    ) -> Result<Response<Self::SearchStream>, Status> {
        let request = request.into_inner();
        let result = self.app.accounts_cache.search(&request).await;
        let accounts = get_accounts_vector(result);
        service_sdk::my_grpc_extensions::grpc_server::send_vec_to_stream(accounts, |x| x).await
    }
    async fn ping(&self, _: tonic::Request<()>) -> Result<tonic::Response<()>, tonic::Status> {
        Ok(tonic::Response::new(()))
    }
}

fn get_accounts_vector(accounts: Option<Vec<Account>>) -> Vec<AccountGrpcModel> {
    match accounts {
        Some(accounts) => accounts
            .iter()
            .map(|x| x.to_owned().into())
            .collect::<Vec<AccountGrpcModel>>(),
        None => vec![],
    }
}
