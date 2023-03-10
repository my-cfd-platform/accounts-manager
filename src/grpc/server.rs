use crate::accounts_manager::accounts_manager_grpc_service_server::AccountsManagerGrpcServiceServer;
use crate::AppContext;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;

#[derive(Clone)]
pub struct GrpcService {
    pub app: Arc<AppContext>,
}

impl GrpcService {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

pub async fn start_grpc_server(app: Arc<AppContext>, port: u16) {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let service = GrpcService::new(app.clone());

    println!("Listening to {:?} as grpc endpoint", addr);

    Server::builder()
        .add_service(AccountsManagerGrpcServiceServer::new(service.clone()))
        .serve(addr)
        .await
        .unwrap();
}
