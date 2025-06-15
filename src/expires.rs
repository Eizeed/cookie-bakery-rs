use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy)]
pub enum Expiration {
    Session,
    DateTime(DateTime<Utc>),
}
