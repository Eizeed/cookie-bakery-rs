use chrono::{DateTime, NaiveDateTime, Utc};
use std::{borrow::Cow, str::FromStr, time::Duration};

#[derive(Debug, Clone)]
pub enum Expires {
    Session,
    DateTime(DateTime<Utc>),
}

#[derive(Debug, Clone, Default)]
pub enum SameSite {
    Strict,
    Lax,
    #[default]
    None,
}

impl TryFrom<&str> for SameSite {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Strict" => Ok(SameSite::Strict),
            "Lax" => Ok(SameSite::Lax),
            "None" => Ok(SameSite::None),
            _ => Err(ParseError::InvalidSameSite),
        }
    }
}

#[derive(Debug, Default)]
pub struct Cookie<'a> {
    name: Cow<'a, str>,
    val: Cow<'a, str>,
    expires: Option<Expires>,
    max_age: Option<Duration>,
    domain: Option<Cow<'a, str>>,
    path: Option<Cow<'a, str>>,
    secure: bool,
    http_only: bool,
    same_site: Option<SameSite>,
}

impl<'a> Cookie<'a> {}

impl<'a> TryFrom<&'a str> for Cookie<'a> {
    type Error = ParseError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut cookie = Cookie::default();

        let mut iter = value.split('=');

        cookie.name = iter.next().ok_or(ParseError::MissingPair)?.trim().into();

        let rest = iter.next().ok_or(ParseError::MissingPair)?.trim();

        let mut iter = rest.split(';');

        cookie.val = iter.next().unwrap().trim().into();

        for atr in iter {
            let mut parts = atr.splitn(2, '=');
            let key = parts.next().unwrap_or("").trim();
            let val = parts.next().map(str::trim);

            match key {
                "Expires" => {
                    cookie.expires = match val {
                        Some(val) => Some(Expires::DateTime(parse_date(val)?)),
                        None => None,
                    }
                }
                "Max-Age" => {
                    cookie.max_age = match val {
                        Some(val) => {
                            let secs: u64 = val.parse().map_err(|_| ParseError::InvalidMaxAge)?;
                            Some(Duration::from_secs(secs))
                        }
                        None => None,
                    }
                }
                "Domain" => cookie.domain = val.map(|s| Cow::Borrowed(s)),
                "Path" => cookie.path = val.map(|s| Cow::Borrowed(s)),
                "Secure" => cookie.secure = true,
                "HttpOnly" => cookie.http_only = true,
                "SameSite" => {
                    cookie.same_site = match val {
                        Some(val) => Some(val.try_into()?),
                        None => None,
                    }
                }
                _ => {}
            }
        }

        Ok(cookie)
    }
}

fn parse_date(str: &str) -> Result<DateTime<Utc>, ParseError> {
    let date = str
        .split("GMT")
        .next()
        .ok_or(ParseError::InvalidDate)?
        .trim();

    let date = NaiveDateTime::from_str(date).map_err(|_| ParseError::InvalidDate)?;

    Ok(DateTime::from_naive_utc_and_offset(date, Utc))
}

pub enum ParseError {
    MissingPair,
    EmptyName,
    InvalidMaxAge,
    InvalidSameSite,
    InvalidDate,
    Utf8Error,
}
