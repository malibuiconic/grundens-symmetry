use serde::{Deserialize, Serialize};
use chrono::{ DateTime, Utc, NaiveDate };


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Count {
    pub count: Option<u32>,
    pub precision: Option<CountPrecision>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum CountPrecision {
    #[serde(rename = "AT_LEAST")]
    ATLEAST,
    EXACT,
}

// Helper Functions:
pub fn convert_naive_date_to_utc_datetime(date: Option<NaiveDate>) -> Option<DateTime<Utc>> {
    date.map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
 }