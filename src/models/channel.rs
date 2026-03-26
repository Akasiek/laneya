use crate::schema::channels;
use diesel::{Insertable, Queryable, Selectable};

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = channels)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Channel {
    pub id: i32,
    pub channel_id: String,
    pub channel_name: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = channels)]
pub struct NewChannel {
    pub channel_id: String,
    pub channel_name: String,
}

