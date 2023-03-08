use std::time::Duration;

use tonic::transport::Channel;

use crate::{
    accounts_manager_persistence::accounts_manager_persistence_grpc_service_client::AccountsManagerPersistenceGrpcServiceClient,
    Account,
};

pub struct AccountsManagerPersistenceGrpcClient {
    channel: Channel,
    timeout: Duration,
}

impl AccountsManagerPersistenceGrpcClient {
    pub async fn new(grpc_address: String) -> Self {
        let channel = Channel::from_shared(grpc_address)
            .unwrap()
            .connect()
            .await
            .unwrap();
        Self {
            channel,
            timeout: Duration::from_secs(1),
        }
    }

    fn create_grpc_service(&self) -> AccountsManagerPersistenceGrpcServiceClient<Channel> {
        let client: AccountsManagerPersistenceGrpcServiceClient<Channel> =
            AccountsManagerPersistenceGrpcServiceClient::new(self.channel.clone());

        client
    }

    pub async fn get_accounts(&self) -> Vec<Account> {
        let mut client = self.create_grpc_service();

        let response = client.get_all_accounts(()).await.unwrap();

        return match my_grpc_extensions::read_grpc_stream::as_vec(
            response.into_inner(),
            self.timeout,
        )
        .await
        .unwrap()
        {
            Some(result) => result.iter().map(|x| x.to_owned().into()).collect(),
            None => vec![],
        };
    }
}
