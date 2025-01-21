use super::*;

#[derive(Debug, Deserialize, Serialize, Table, FromPgRow, Clone)]
#[table_name("offices")]
pub struct Office {
    pub id: uuid::Uuid,
    pub neighborhood: String,
    pub street: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Updatable)]
pub struct UpdateOffice {
    pub street: Option<String>,
    pub neighborhood: Option<String>,
    pub description: Option<String>,
}
