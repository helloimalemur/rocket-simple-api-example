use serde::*;

#[derive(Debug, Ord, PartialOrd, Clone,PartialEq, Eq, Deserialize, Serialize,)]
pub struct Login {
    pub username: String,
    pub password: String,
    pub ipaddress: String,
}
