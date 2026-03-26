use crate::schema::channels;
use diesel::{Queryable, Selectable};

#[derive(Queryable, Selectable)]
#[diesel(table_name = channels)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[derive(Debug)]
pub struct Channel {
    pub id: i32,
    pub channel_id: String,
    pub channel_name: String,
    pub created_at: chrono::NaiveDateTime,
}
