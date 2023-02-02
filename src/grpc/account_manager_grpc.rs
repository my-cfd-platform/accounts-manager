use std::pin::Pin;

use uuid::Uuid;

use crate::{
    accounts_manager::{
        accounts_manager_grpc_service_server::AccountsManagerGrpcService, AccountGrpcModel,
        AccountManagerCreateAccountGrpcRequest, AccountManagerGetClientAccountGrpcRequest,
        AccountManagerGetClientAccountGrpcResponse, AccountManagerGetClientAccountsGrpcRequest,
        AccountManagerUpdateAccountBalanceGrpcRequest,
        AccountManagerUpdateAccountBalanceGrpcResponse, AccountManagerUpdateTradingDisabledGrpcRequest, AccountManagerUpdateTradingDisabledGrpcResponse,
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
        // let my_telemetry =
        //     my_grpc_extensions::get_telemetry(&request.metadata(), request.remote_addr(), "swap");

        let request = request.into_inner();

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
            trading_group: self.app.settings.default_account_trading_group.clone(),
        };

        self.app
            .accounts_cache
            .add_account(account_to_insert.clone())
            .await;

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
            Some(accounts) => accounts,
            None => vec![],
        };

        my_grpc_extensions::grpc_server::send_vec_to_stream(accounts, |x| x.into()).await
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

        let response = match update_balance_result{
            Ok(account) => AccountManagerUpdateAccountBalanceGrpcResponse{
                result: 0,
                account: Some(account.into()),
            },
            Err(error) => AccountManagerUpdateAccountBalanceGrpcResponse{
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

        let response = match update_balance_result{
            Ok(account) => AccountManagerUpdateTradingDisabledGrpcResponse{
                result: 0,
                account: Some(account.into()),
            },
            Err(error) => AccountManagerUpdateTradingDisabledGrpcResponse{
                result: error.as_grpc_error(),
                account: None,
            },
        };

        Ok(tonic::Response::new(response))
    }

}
