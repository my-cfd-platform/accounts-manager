mod app;
mod background;
mod caches;
mod grpc;
mod grpc_client;
mod settings;
mod flows;

pub mod accounts_manager {
    tonic::include_proto!("accounts_manager");
}

pub mod accounts_manager_persistence {
    tonic::include_proto!("accounts_manager_persistence");
}

pub use app::*;
pub use flows::*;
pub use background::*;
pub use caches::*;
pub use grpc::*;
pub use grpc_client::*;
pub use settings::*;
