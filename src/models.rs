use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Sample {
    uuid: Uuid,
    date_time: DateTime<Utc>,
    moisture: f64,
    humidity: f64,
    temperature: f64
}
