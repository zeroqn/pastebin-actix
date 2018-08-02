use std::time::SystemTime;

use crate::models::schema::pastes;

#[derive(Queryable, Debug, PartialEq, Serialize, Deserialize)]
pub struct Paste {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub created_at: SystemTime,
    pub modified_at: SystemTime,
}

#[derive(Insertable)]
#[table_name = "pastes"]
pub struct NewPaste<'a> {
    pub title: &'a str,
    pub body: &'a str,
    pub created_at: &'a SystemTime,
    pub modified_at: &'a SystemTime,
}
