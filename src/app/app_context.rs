use std::sync::Arc;

use rust_extensions::AppStates;

use crate::{AccountsCache, SettingsModel};

pub struct AppContext {
    pub accounts_cache: Arc<AccountsCache>,
    pub settings: Arc<SettingsModel>,
    pub app_states: Arc<AppStates>,
}

impl AppContext {
    pub fn new(settings: Arc<SettingsModel>) -> Self {
        Self {
            accounts_cache: Arc::new(AccountsCache::new()),
            settings,
            app_states: Arc::new(AppStates::create_initialized())
        }
    }
}
