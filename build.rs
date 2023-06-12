fn main() {
    let url = "https://raw.githubusercontent.com/my-cfd-platform/proto-files/main/proto/";
    ci_utils::sync_and_build_proto_file(url, "AccountsManagerGrpcService.proto");
    ci_utils::sync_and_build_proto_file(url, "AccountsManagerPersistenceGrpcService.proto");
    //tonic_build::compile_protos(format!("proto{}{}", std::path::MAIN_SEPARATOR, "AccountsManagerGrpcService.proto").as_str()).unwrap();
}
