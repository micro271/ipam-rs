use super::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Office {
    pub address: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOffice {
    pub description: Option<String>,
    pub address: Option<String>,
}
