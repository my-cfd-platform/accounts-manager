use std::sync::Arc;

use accounts_manager::{
    accounts_manager::accounts_manager_grpc_service_server::AccountsManagerGrpcServiceServer,
    AppContext, GrpcService, SettingsReader
};

#[tokio::main]
async fn main() {
    let settings_reader = SettingsReader::new(".my-cfd-platform").await;
    let settings_reader = Arc::new(settings_reader);

    let mut service_context = service_sdk::ServiceContext::new(settings_reader.clone()).await;

    let app_context = Arc::new(AppContext::new(settings_reader, &service_context).await);

    service_context.configure_grpc_server(|config| {
        config.add_grpc_service(AccountsManagerGrpcServiceServer::new(GrpcService::new(
            app_context.clone(),
        )))
    });

    service_context.start_application().await;
}
