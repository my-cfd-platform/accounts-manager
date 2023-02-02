FROM rust:slim
COPY ./target/release/accounts-manager ./target/release/accounts-manager
ENTRYPOINT ["./target/release/accounts-manager"]