use service_sdk::{async_trait, my_grpc_extensions, my_telemetry};
#[service_sdk::my_grpc_extensions::client::generate_grpc_client(
    proto_file = "./proto/AccountsManagerPersistenceGrpcService.proto",
    crate_ns: "crate::accounts_manager_persistence",
    retries: 3,
    request_timeout_sec: 1,
    ping_timeout_sec: 1,
    ping_interval_sec: 3,
)]
pub struct AccountsManagerPersistenceGrpcClient;
