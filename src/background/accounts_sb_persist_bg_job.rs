use std::sync::Arc;

use engine_sb_contracts::AccountPersistEvent;
use rust_extensions::MyTimerTick;

use crate::AppContext;

pub struct AccountsSbPersistBgJob {
    app: Arc<AppContext>,
}

impl AccountsSbPersistBgJob {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for AccountsSbPersistBgJob {
    async fn tick(&self) {
        let messages_to_publish = self
            .app
            .accounts_persist_queue
            .dequeue_all()
            .await
            .iter()
            .map(|x| match x {
                crate::PersistAccountQueueItem::CreateAccount(account) => AccountPersistEvent {
                    add_account_event: Some(account.to_owned().into()),
                    update_account_event: None,
                },
                crate::PersistAccountQueueItem::UpdateAccount(account) => AccountPersistEvent {
                    add_account_event: None,
                    update_account_event: Some(account.to_owned().into()),
                },
            })
            .collect::<Vec<AccountPersistEvent>>();

        self.app
            .account_persist_events_publisher
            .publish_messages(&messages_to_publish)
            .await
            .unwrap();

        print!("Done publishing messages. Len: {}", messages_to_publish.len());
    }
}
