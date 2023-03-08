mod app;
mod background;
mod caches;
mod grpc;

pub mod accounts_manager {
    tonic::include_proto!("accounts_manager");
}

pub mod accounts_manager_persistence {
    tonic::include_proto!("accounts_manager_persistence");
}

pub use app::*;
pub use background::*;
pub use caches::*;
pub use grpc::*;
