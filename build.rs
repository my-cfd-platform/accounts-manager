fn main() {
    tonic_build::compile_protos("proto/accounts_manager_grcp_service.proto").unwrap();
    tonic_build::compile_protos("proto/accounts_manager_persistence_grcp_service.proto").unwrap();
}
