use std::{pin::Pin, vec};

use engine_sb_contracts::AccountPersistEvent;
use tonic::Request;
use uuid::Uuid;

use crate::{
    accounts_manager::{
        accounts_manager_grpc_service_server::AccountsManagerGrpcService, AccountGrpcModel,
        AccountManagerCreateAccountGrpcRequest, AccountManagerGetClientAccountGrpcRequest,
        AccountManagerGetClientAccountGrpcResponse, AccountManagerGetClientAccountsGrpcRequest,
        AccountManagerUpdateAccountBalanceGrpcRequest,
        AccountManagerUpdateAccountBalanceGrpcResponse,
        AccountManagerUpdateTradingDisabledGrpcRequest,
        AccountManagerUpdateTradingDisabledGrpcResponse,
    },
    Account,
};

use super::server::GrpcService;

#[tonic::async_trait]
impl AccountsManagerGrpcService for GrpcService {
    type GetClientAccountsStream = Pin<
        Box<
            dyn tonic::codegen::futures_core::Stream<Item = Result<AccountGrpcModel, tonic::Status>>
                + Send
                + Sync
                + 'static,
        >,
    >;

    async fn create_account(
        &self,
        request: tonic::Request<AccountManagerCreateAccountGrpcRequest>,
    ) -> Result<tonic::Response<AccountGrpcModel>, tonic::Status> {
        let request = request.into_inner();

        let tg = match request.trading_group_id {
            Some(tg) => tg,
            None => self.app.settings.default_account_trading_group.clone(),
        };

        let date = chrono::offset::Utc::now().timestamp_millis() as u64;
        let account_to_insert = Account {
            id: Uuid::new_v4().to_string(),
            balance: self.app.settings.default_account_balance,
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
            .publish(&AccountPersistEvent {
                add_account_event: Some(account.clone().into()),
                update_account_event: None,
            })
            .await
            .unwrap();

        return Ok(tonic::Response::new(account_to_insert.into()));
    }

    async fn get_client_account(
        &self,
        request: tonic::Request<AccountManagerGetClientAccountGrpcRequest>,
    ) -> Result<tonic::Response<AccountManagerGetClientAccountGrpcResponse>, tonic::Status> {
        let AccountManagerGetClientAccountGrpcRequest {
            trader_id,
            account_id,
        } = request.into_inner();
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

    async fn get_client_accounts(
        &self,
        request: tonic::Request<AccountManagerGetClientAccountsGrpcRequest>,
    ) -> Result<tonic::Response<Self::GetClientAccountsStream>, tonic::Status> {
        let AccountManagerGetClientAccountsGrpcRequest { trader_id } = request.into_inner();
        let accounts = self.app.accounts_cache.get_accounts(&trader_id).await;

        let accounts = match accounts {
            Some(accounts) => accounts
                .iter()
                .map(|x| x.to_owned().into())
                .collect::<Vec<AccountGrpcModel>>(),
            None => match &self.app.settings.accounts_default_currency {
                Some(currency) => {
                    let request = AccountManagerCreateAccountGrpcRequest {
                        trader_id: trader_id.clone(),
                        currency: currency.clone(),
                        process_id: Uuid::new_v4().to_string(),
                        trading_group_id: None
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

        my_grpc_extensions::grpc_server::send_vec_to_stream(accounts, |x| x).await
    }

    async fn update_client_account_balance(
        &self,
        request: tonic::Request<AccountManagerUpdateAccountBalanceGrpcRequest>,
    ) -> Result<tonic::Response<AccountManagerUpdateAccountBalanceGrpcResponse>, tonic::Status>
    {
        let request = request.into_inner();

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
                    .publish(&AccountPersistEvent {
                        add_account_event: None,
                        update_account_event: Some(account.clone().into()),
                    })
                    .await
                    .unwrap();

                AccountManagerUpdateAccountBalanceGrpcResponse {
                    result: 0,
                    account: Some(account.into()),
                }
            }
            Err(error) => AccountManagerUpdateAccountBalanceGrpcResponse {
                result: error.as_grpc_error(),
                account: None,
            },
        };

        Ok(tonic::Response::new(response))
    }

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
                    .publish(&AccountPersistEvent {
                        add_account_event: None,
                        update_account_event: Some(account.clone().into()),
                    })
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
}
