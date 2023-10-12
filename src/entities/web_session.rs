use serde::*;

#[derive(Debug, Ord, PartialOrd, Clone,PartialEq, Eq, Deserialize, Serialize,)]
pub struct WebSession {
    pub user_name: String,
    pub session_id: String,
    pub date_created: String,
}
