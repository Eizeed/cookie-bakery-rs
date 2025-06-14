use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub enum Expires {
    Session,
    DateTime(DateTime<Utc>),
}
