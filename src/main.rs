use std::sync::Arc;

use accounts_manager::{start_grpc_server, AppContext, SettingsReader};

#[tokio::main]
async fn main() {
    let settings_reader = SettingsReader::new(".yourfin").await;
    let settings_model = Arc::new(settings_reader.get_settings().await);
    let app = Arc::new(AppContext::new(settings_model).await);
    // let mut sb_queue_processing_timer = MyTimer::new(Duration::from_secs(2));
    // let mut queue_persist_timer = MyTimer::new(Duration::from_secs(2));

    // sb_queue_processing_timer.register_timer(
    //     "Sb queue processing timer",
    //     Arc::new(AccountsSbPersistBgJob::new(app.clone())),
    // );
    // queue_persist_timer.register_timer(
    //     "Sb queue processing timer",
    //     Arc::new(PersistSbQueueJob::new(app.clone())),
    // );

    // sb_queue_processing_timer.start(app.app_states.clone(), my_logger::LOGGER.clone());
    // queue_persist_timer.start(app.app_states.clone(), my_logger::LOGGER.clone());

    app.sb_client.start().await;
    start_grpc_server(app.clone(), 8888).await;
    app.app_states.wait_until_shutdown().await;
}
