// 应用全局状态

use std::sync::Mutex;
use crate::auth::AuthState;
use crate::account::AccountStore;

#[derive(Clone)]
pub struct PendingLogin {
    pub provider: String,
    pub code_verifier: String,
    pub state: String,
    pub machineid: String,
}

pub struct AppState {
    pub store: Mutex<AccountStore>,
    pub auth: AuthState,
    pub pending_login: Mutex<Option<PendingLogin>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            store: Mutex::new(AccountStore::default()),
            auth: AuthState::default(),
            pending_login: Mutex::new(None),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
}
