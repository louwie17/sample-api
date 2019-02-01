use actix::prelude::*;
use failure::Error;
use r2d2;
use r2d2_sqlite;

use uuid::Uuid;
use std::collections::{ HashMap };
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

#[derive(Debug)]
pub struct SampleQuery {
    pub query_string: String
}

impl Message for SampleQuery {
    type Result = Result<Vec<Sample>, Error>;
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

impl Handler<SampleQuery> for DbExecutor {
    type Result = Result<Vec<Sample>, Error>;

    fn handle(&mut self, msg: SampleQuery, _: &mut Self::Context) -> Self::Result {
        let conn: Connection = self.0.get()?;

        return query_samples(conn, msg);
    }
}

fn query_samples(conn: Connection, msg: SampleQuery) -> Result<Vec<Sample>, Error> {
    println!("Params ${:?}", msg);
    let mut stmt = vec!["
    SELECT moisture, humidity, temperature, DateTime, uuid FROM samples "];
    let mut mymap = HashMap::new();
    msg.query_string.split('&').for_each(|x| {
        let items: Vec<&str> = x.split('=').collect();
        mymap.insert(items[0], items[1]);
    });
    
    if (mymap.get("from").is_some() && mymap.get("to").is_some()) {
        stmt.extend_from_slice(&[r#" WHERE DateTime BETWEEN ""#, mymap.get("from").unwrap(), r#"" AND ""#, mymap.get("to").unwrap(), r#"" "#]);
    }
    if (mymap.get("limit").is_some()) {
        stmt.extend_from_slice(&[" LIMIT ", mymap.get("limit").unwrap()]);
    }
    stmt.extend_from_slice(&["ORDER BY DateTime DESC;"]);
    let mut stmts = &stmt[..].iter().fold(String::new(), |acc, r| {
                            acc + r + ""
                        });
    println!("Stmt ${:?}", stmts);

    let mut prep_stmt = conn.prepare(stmts)?;
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