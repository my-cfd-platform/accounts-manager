fn main() {
    let url = "https://raw.githubusercontent.com/my-cfd-platform/proto-files/main/proto/";
    ci_utils::sync_and_build_proto_file_with_builder(url, "AccountsManagerGrpcService.proto", |x| {
        x.type_attribute(
            ".",
            "#[derive(serde::Serialize,serde::Deserialize)]"
        )
    });
    ci_utils::sync_and_build_proto_file_with_builder(url, "AccountsManagerPersistenceGrpcService.proto", |x| {
        x.type_attribute(
            ".",
            "#[derive(serde::Serialize,serde::Deserialize)]"
        )
    });
}
