use crate::ApiV1;
use std::sync::OnceLock;

static CURRENT_CONTEXT: OnceLock<WSLContext> = OnceLock::new();

pub struct WSLContext {
    pub api: ApiV1<'static>,
}

impl WSLContext {
    pub fn get_current() -> Option<&'static WSLContext> {
        CURRENT_CONTEXT.get()
    }

    pub fn get_current_or_panic() -> &'static WSLContext {
        Self::get_current().expect("WSL context is not initialised.")
    }

    pub fn init(api: ApiV1<'static>) -> Option<&'static Self> {
        CURRENT_CONTEXT.set(WSLContext { api }).ok()?;
        CURRENT_CONTEXT.get()
    }
}
