use std::time::Duration;
use std::{borrow::Cow, fmt::Display};

use crate::{builder::CookieBuilder, expires::Expires, same_site::SameSite};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CookieStr<'a> {
    Indexed(usize, usize),
    Concrete(Cow<'a, str>),
}

impl<'a> CookieStr<'a> {
    pub fn indexed(needle: &str, haystack: &str) -> Option<CookieStr<'static>> {
        let haystack_ptr = haystack.as_ptr() as usize;
        let needle_ptr = needle.as_ptr() as usize;

        if needle_ptr < haystack_ptr {
            return None;
        }

        if (needle_ptr + needle.len()) > (haystack_ptr + haystack.len()) {
            return None;
        }

        let start = needle_ptr - haystack_ptr;
        let end = start + needle.len();

        Some(CookieStr::Indexed(start, end))
    }
}

#[derive(Debug)]
pub struct Cookie<'a> {
    pub(crate) cookie_string: Option<Cow<'a, str>>,
    pub(crate) name: CookieStr<'a>,
    pub(crate) val: CookieStr<'a>,
    pub(crate) expires: Option<Expires>,
    pub(crate) max_age: Option<Duration>,
    pub(crate) domain: Option<CookieStr<'a>>,
    pub(crate) path: Option<CookieStr<'a>>,
    pub(crate) secure: Option<bool>,
    pub(crate) http_only: Option<bool>,
    pub(crate) same_site: Option<SameSite>,
}

impl<'a> Cookie<'a> {
    pub fn builder() -> CookieBuilder<'a> {
        CookieBuilder::from(Cookie::default())
    }
}

impl<'a> Display for Cookie<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.name, self.val)?;

        if self.expires.is_some() {
            match self.expires.as_ref().unwrap() {
                Expires::DateTime(date) => {
                    write!(f, "; Expires={} GMT", date.format("%a, %d %b %Y %H:%M:%S"))?;
                }
                Expires::Session => {}
            }
        }
        if self.max_age.is_some() {
            write!(f, "; Max-Age={}", self.max_age.as_ref().unwrap().as_secs())?;
        }
        if self.domain.is_some() {
            write!(f, "; Domain={}", self.domain.as_ref().unwrap())?;
        }
        if self.path.is_some() {
            write!(f, "; Path={}", self.path.as_ref().unwrap())?;
        }
        if self.secure {
            write!(f, "; Secure")?;
        }
        if self.http_only {
            write!(f, "; HttpOnly")?;
        }
        if self.same_site.is_some() {
            write!(f, "; SameSite={:?}", self.same_site.as_ref().unwrap())?;
        }

        Ok(())
    }
}
