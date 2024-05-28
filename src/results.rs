use crate::probe::AuthState;

#[derive(Debug, Default)]
pub struct Results {
    pub auths_initially: Option<bool>,
    pub auth: AuthState,
    pub authentication_works: Option<bool>,
    pub benign_fetch: Option<bool>,
}
