use std::{borrow::Cow, time::Duration};

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::{Cookie, cookie::CookieStr, expires::Expires, same_site::SameSite};

const FMT1: &'static str = "%a, %d %b %Y %H:%M:%S GMT";
const FMT2: &'static str = "%A, %d-%b-%y %H:%M:%S GMT";
const FMT3: &'static str = "%a, %b %-d %H:%M:%S %-Y"; // Can't handle padding with spaces. Only with zeroes or nothing
const FMT4: &'static str = "%a, %d-%b-%-Y %H:%M:%S GMT";

pub type ParseResult<T> = Result<T, ParseError>;
pub fn parse_cookie<'a, T: Into<Cow<'a, str>>>(s: T) -> ParseResult<Cookie<'a>> {
    let str = s.into();
    let mut cookie = parse_inner(&str)?;

    cookie.cookie_string = Some(str);

    Ok(cookie)
}

fn parse_inner<'a>(s: &str) -> Result<Cookie<'a>, ParseError> {
    let mut attributes = s.split(';');

    let name_val = attributes.next().expect("Name and Value are Some");

    let (name, val) = match name_val.find('=') {
        Some(idx) => (name_val[..idx].trim(), name_val[(idx + 1)..].trim()),
        None => return Err(ParseError::MissingPair),
    };

    if name.is_empty() {
        return Err(ParseError::EmptyName);
    }

    let name = CookieStr::indexed(name, s).expect("Name in bounds of s");
    let val = CookieStr::indexed(val, s).expect("Val in bounds of s");

    let mut cookie = Cookie {
        cookie_string: None,
        name,
        val,
        expires: None,
        max_age: None,
        domain: None,
        path: None,
        secure: None,
        http_only: None,
        same_site: None,
    };

    for attr in attributes {
        let (key, val) = match attr.find('=') {
            Some(idx) => (attr[0..idx].trim(), Some(attr[(idx + 1)..].trim())),
            None => (attr.trim(), None),
        };

        match (key, val) {
            ("Expires", Some(expires)) => {
                cookie.expires = Some(Expires::DateTime(parse_date_with_all_formats(expires)?))
            }
            ("Max-Age", Some(max_age)) => {
                cookie.max_age = {
                    let is_negatove = max_age.starts_with('-');
                    let max_age = if is_negatove { &max_age[1..] } else { max_age };

                    if !max_age.chars().all(|c| c.is_digit(10)) {
                        continue;
                    }

                    if is_negatove {
                        Some(Duration::ZERO)
                    } else {
                        Some(
                            max_age
                                .parse::<u64>()
                                .map(Duration::from_secs)
                                .unwrap_or_else(|_| Duration::from_secs(u64::MAX)),
                        )
                    }
                }
            }
            ("Domain", Some(domain)) => {
                cookie.domain = Some(CookieStr::indexed(domain, s).expect("Domain in bounds of s"))
            }
            ("Path", Some(path)) => {
                cookie.path = Some(CookieStr::indexed(path, s).expect("Path in bounds of s"))
            }
            ("Secure", _) => cookie.secure = Some(true),
            ("HttpOnly", _) => cookie.http_only = Some(true),
            ("SameSite", Some(same_site)) => {
                if same_site.eq_ignore_ascii_case("strict") {
                    cookie.same_site = Some(SameSite::Strict)
                } else if same_site.eq_ignore_ascii_case("lax") {
                    cookie.same_site = Some(SameSite::Lax)
                } else if same_site.eq_ignore_ascii_case("none") {
                    cookie.same_site = Some(SameSite::None)
                } else {
                    // Do nothing
                }
            }
            _ => {}
        }
    }

    Ok(cookie)
}

fn parse_date(str: &str) -> Result<DateTime<Utc>, ParseError> {
    let date = str
        .split("GMT")
        .next()
        .ok_or(ParseError::InvalidDate)?
        .trim();

    let date = NaiveDateTime::parse_from_str(date, "%a, %d %b %Y %H:%M:%S")
        .map_err(|_| ParseError::InvalidDate)?;

    Ok(DateTime::from_naive_utc_and_offset(date, Utc))
}

fn parse_date_with_all_formats(str: &str) -> Result<DateTime<Utc>, ParseError> {
    let date = str
        .split("GMT")
        .next()
        .ok_or(ParseError::InvalidDate)?
        .trim();

    let date = NaiveDateTime::parse_from_str(date, FMT1)
        .or_else(|_| NaiveDateTime::parse_from_str(date, FMT2))
        .or_else(|_| NaiveDateTime::parse_from_str(date, FMT3))
        .or_else(|_| NaiveDateTime::parse_from_str(date, FMT4))
        .map_err(|_| ParseError::InvalidDate);

    date.map(|d| DateTime::from_naive_utc_and_offset(d, Utc))
}

#[derive(Debug, Clone)]
pub enum ParseError {
    MissingPair,
    EmptyName,
    InvalidMaxAge,
    InvalidSameSite,
    InvalidDate,
    Utf8Error,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn parse_valid() {
        let date1 = "Tue, 21 Oct 2025 07:28:00 GMT";
        let date2 = "Mon, 14 Jun 2027 12:00:00 GMT";
        let date3 = "Thu, 01 Jan 2026 23:59:59 GMT";

        let res1 = parse_date(date1).unwrap();
        let res2 = parse_date(date2).unwrap();
        let res3 = parse_date(date3).unwrap();

        assert_eq!(
            res1,
            DateTime::<Utc>::from_str("2025-10-21T07:28:00Z").unwrap()
        );
        assert_eq!(
            res2,
            DateTime::<Utc>::from_str("2027-06-14T12:00:00Z").unwrap()
        );
        assert_eq!(
            res3,
            DateTime::<Utc>::from_str("2026-01-01T23:59:59Z").unwrap()
        );
    }

    #[test]
    fn parse_invalid() {
        let invalid_day_of_week = "Wed, 21 Oct 2025 07:28:00 GMT";
        let invalid_format = "21 Oct 2025 07:28:00";
        let invalid_tz = "Thu, 01 Jan 2026 23:59:59 UTC";
        let invalid_time = "Thu, 01 Jan 2026 24:59:59 GMT";

        assert!(parse_date(invalid_day_of_week).is_err());
        assert!(parse_date(invalid_format).is_err());
        assert!(parse_date(invalid_tz).is_err());
        assert!(parse_date(invalid_time).is_err());
    }

    #[test]
    fn cookie() {
        let cookie = "sessionId=abc123; Expires=Tue, 21 Oct 2025 07:28:00 GMT; Max-Age=3600; Domain=example.com; Path=/; Secure; HttpOnly; SameSite=Strict";
        let cookie = Cookie::try_from(cookie).unwrap();
        let cookie = "authToken=xyz789; Expires=Fri, 01 Jan 2027 12:00:00 GMT; Max-Age=7200; Domain=example.org; Path=/account; Secure; HttpOnly; SameSite=Lax";
        let cookie = Cookie::try_from(cookie).unwrap();
        println!("{cookie:#?}");
        println!("{cookie}");
    }
}
