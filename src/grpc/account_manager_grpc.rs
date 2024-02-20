use std::{pin::Pin, vec};

use crate::accounts_manager::{
    AccountManagerGetAccountsByGroupGrpcRequest, AccountManagerGetTraderIdByAccountIdGrpcRequest,
    AccountManagerGetTraderIdByAccountIdGrpcResponse, AccountManagerUpdateTradingGroupGrpcRequest,
    AccountsManagerOperationResult, SearchAccounts,
};
use crate::update_balance;
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
    AccountBalanceUpdateSbModel, AccountPersistEvent,
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
    type GetTradingGroupAccountsStream = Pin<
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
            .publish_with_headers(
                &AccountPersistEvent {
                    add_account_event: Some(account.clone().into()),
                    update_account_event: None,
                },
                vec![(
                    "type".to_string(),
                    self.app.settings_reader.get_env_type().await,
                )]
                .into(),
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
        let request = request.into_inner();
        let AccountManagerGetClientAccountGrpcRequest {
            trader_id,
            account_id,
        } = request;

        let account = self
            .app
            .accounts_cache
            .get_account(&trader_id, &account_id)
            .await;

        return Ok(tonic::Response::new(account.into()));
    }

    #[with_telemetry]
    async fn get_client_accounts(
        &self,
        request: tonic::Request<AccountManagerGetClientAccountsGrpcRequest>,
    ) -> Result<tonic::Response<Self::GetClientAccountsStream>, tonic::Status> {
        let request = request.into_inner();
        let AccountManagerGetClientAccountsGrpcRequest { trader_id } = request;
        let accounts = self
            .app
            .accounts_cache
            .get_accounts(&trader_id)
            .await
            .map(|x| {
                x.into_iter()
                    .map(|acc| acc.into())
                    .collect::<Vec<AccountGrpcModel>>()
            });

        let accounts_to_send = match accounts {
            Some(accounts) => accounts,
            None => {
                if let Some(def_currency) = self
                    .app
                    .settings_reader
                    .get_accounts_default_currency()
                    .await
                {
                    let request = AccountManagerCreateAccountGrpcRequest {
                        trader_id: trader_id.clone(),
                        currency: def_currency.clone(),
                        process_id: Uuid::new_v4().to_string(),
                        trading_group_id: None,
                    };

                    let account = self
                        .create_account(Request::new(request))
                        .await
                        .unwrap()
                        .into_inner();

                    vec![account]
                } else {
                    vec![]
                }
            }
        };

        return service_sdk::my_grpc_extensions::grpc_server::send_vec_to_stream(
            accounts_to_send.into_iter(),
            |x| x,
        )
        .await;
    }

    #[with_telemetry]
    async fn get_trading_group_accounts(
        &self,
        request: tonic::Request<AccountManagerGetAccountsByGroupGrpcRequest>,
    ) -> Result<tonic::Response<Self::GetTradingGroupAccountsStream>, tonic::Status> {
        let request = request.into_inner();
        let AccountManagerGetAccountsByGroupGrpcRequest { trading_group } = request;
        let accounts = self
            .app
            .accounts_cache
            .get_accounts_by_trading_group(&trading_group)
            .await;

        let accounts = match accounts {
            Some(accounts) => accounts,
            None => vec![],
        };

        service_sdk::my_grpc_extensions::grpc_server::send_vec_to_stream(
            accounts.into_iter(),
            |x| x.into(),
        )
        .await
    }

    #[with_telemetry]
    async fn update_client_account_balance(
        &self,
        request: tonic::Request<AccountManagerUpdateAccountBalanceGrpcRequest>,
    ) -> Result<tonic::Response<AccountManagerUpdateAccountBalanceGrpcResponse>, tonic::Status>
    {
        let request = request.into_inner();
        let transaction_id = Uuid::new_v4().to_string();

        trade_log::trade_log!(
            &request.trader_id,
            &request.account_id,
            &request.process_id,
            &transaction_id,
            "Got update balance request.",
            my_telemetry.clone(),
            "request" = &request
        );

        if let Some(response) = self.app.cache.get(&request.process_id).await {
            trade_log::trade_log!(
                &request.trader_id,
                &request.account_id,
                &request.process_id,
                &transaction_id,
                "Found request with same process id - returning error.",
                my_telemetry.clone(),
                "request" = &request,
                "previous_response" = &response
            );

            return Ok(tonic::Response::new(
                AccountManagerUpdateAccountBalanceGrpcResponse {
                    result: AccountsManagerOperationResult::ProcessIdDuplicate as i32,
                    update_balance_info: None,
                },
            ));
        }

        let update_balance_result =
            update_balance(&self.app, &request, transaction_id.clone(), &my_telemetry).await;

        trade_log::trade_log!(
            &request.trader_id,
            &request.account_id,
            &request.process_id,
            &transaction_id,
            "Executed update balance request.",
            my_telemetry.clone(),
            "request" = &request,
            "result" = &update_balance_result
        );

        let response = match update_balance_result{
            Ok(account) => AccountManagerUpdateAccountBalanceGrpcResponse {
                result: 0,
                update_balance_info: Some(AccountManagerUpdateBalanceBalanceGrpcInfo {
                    account: Some(account.into()),
                    operation_id: transaction_id.clone(),
                }),
            },
            Err(error) => AccountManagerUpdateAccountBalanceGrpcResponse {
                result: error.as_grpc_error(),
                update_balance_info: None,
            },
        };

        self.app
            .cache
            .set(&request.process_id, response.clone())
            .await;

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

        trade_log::trade_log!(
            &request.trader_id,
            &request.account_id,
            &request.process_id,
            "",
            "Got update trading disabled request.",
            my_telemetry.clone(),
            "request" = &request
        );
        let response = match update_balance_result {
            Ok(account) => {
                self.app
                    .account_persist_events_publisher
                    .publish_with_headers(
                        &AccountPersistEvent {
                            add_account_event: None,
                            update_account_event: Some(AccountBalanceUpdateSbModel {
                                account_after_update: Some(account.clone().into()),
                                operation: None,
                            }),
                        },
                        vec![(
                            "type".to_string(),
                            self.app.settings_reader.get_env_type().await,
                        )]
                        .into(),
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

        trade_log::trade_log!(
            &request.trader_id,
            &request.account_id,
            &request.process_id,
            "",
            "Got update trading group request.",
            my_telemetry.clone(),
            "request" = &request
        );

        let response = match update_balance_result {
            Ok(account) => {
                self.app
                    .account_persist_events_publisher
                    .publish_with_headers(
                        &AccountPersistEvent {
                            add_account_event: None,
                            update_account_event: Some(AccountBalanceUpdateSbModel {
                                account_after_update: Some(account.clone().into()),
                                operation: None,
                            }),
                        },
                        vec![(
                            "type".to_string(),
                            self.app.settings_reader.get_env_type().await,
                        )]
                        .into(),
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
        service_sdk::my_grpc_extensions::grpc_server::send_vec_to_stream(
            accounts.into_iter(),
            |x| x,
        )
        .await
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
