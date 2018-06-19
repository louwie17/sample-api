use actix::prelude::*;
use failure::Error;
use r2d2;
use r2d2_sqlite;

use uuid::Uuid;
use rusqlite::types::{ValueRef, FromSql, FromSqlResult, FromSqlError};

#[derive(Debug, Serialize, Deserialize)]
pub struct Sample {
    uuid: String,
    date_time: String,
    moisture: f64,
    humidity: f64,
    temperature: f64
}

struct FromSqlUuid(Uuid);

impl FromSql for FromSqlUuid {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        let bytes = value.as_blob()?;
        let uuid = Uuid::from_bytes(bytes).map_err(|err| FromSqlError::Other(Box::new(err)))?;
        Ok(FromSqlUuid(uuid))
    }
}

pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;


pub struct DbExecutor(pub Pool);
impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

pub enum Queries {
    GetAllSamples,
}

impl Message for Queries {
    type Result = Result<Vec<Sample>, Error>;
}
impl Handler<Queries> for DbExecutor {
    type Result = Result<Vec<Sample>, Error>;

    fn handle(&mut self, msg: Queries, _: &mut Self::Context) -> Self::Result {
        let conn: Connection = self.0.get()?;

        match msg {
            Queries::GetAllSamples => get_all_samples(conn)
        }
    }
}

fn get_all_samples(conn: Connection) -> Result<Vec<Sample>, Error> {
    let stmt = "
    SELECT moisture, humidity, temperature, DateTime, uuid FROM samples;";

    let mut prep_stmt = conn.prepare(stmt)?;
    let annuals = prep_stmt.query_map(&[], |row| {
        let uuid = row.get::<_, FromSqlUuid>(4).0;
        Sample {
            moisture: row.get(0),
            humidity: row.get(1),
            temperature: row.get(2),
            date_time: row.get(3),
            uuid: uuid.hyphenated().to_string(),
        }
    })
    .and_then(|mapped_rows| 
        Ok(mapped_rows.map(|row| row.unwrap()).collect::<Vec<Sample>>()))?;

    Ok(annuals)
}