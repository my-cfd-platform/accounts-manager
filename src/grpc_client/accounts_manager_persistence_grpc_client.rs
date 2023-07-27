use my_grpc_client_macros::generate_grpc_client;
use my_grpc_extensions::GrpcChannel;
use my_telemetry::MyTelemetryContext;

use crate::Account;

#[generate_grpc_client(
    proto_file = "./proto/AccountsManagerPersistenceGrpcService.proto",
    crate_ns: "crate::accounts_manager_persistence",
    retries: 3,
    request_timeout_sec: 1,
    ping_timeout_sec: 1,
    ping_interval_sec: 3,
)]
pub struct AccountsManagerPersistenceGrpcClient {
    channel: GrpcChannel<TGrpcService>,
}

impl AccountsManagerPersistenceGrpcClient {
    pub async fn get_accounts(&self) -> Vec<Account> {
        let response = self
            .get_all_accounts((), &MyTelemetryContext::new())
            .await
            .unwrap();

        return match response {
            Some(result) => result.iter().map(|x| x.to_owned().into()).collect(),
            None => vec![],
        };
    }
}

/*
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
 */
