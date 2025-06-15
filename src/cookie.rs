use std::time::Duration;
use std::{borrow::Cow, fmt::Display};

use crate::parse::{ParseError, parse_cookie};
use crate::{builder::CookieBuilder, expires::Expiration, same_site::SameSite};

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

    fn as_str<'s>(&'s self, source: Option<&'s Cow<str>>) -> &'s str {
        match *self {
            CookieStr::Indexed(i, j) => {
                let str =
                    source.expect("Source str must be `Some` when converting indexed str to str");
                &str[i..j]
            }
            CookieStr::Concrete(ref concrete_str) => &*concrete_str,
        }
    }

    fn to_raw_str<'s, 'b: 's>(&'s self, source: &'s Cow<'b, str>) -> Option<&'s str> {
        match *self {
            CookieStr::Indexed(i, j) => match source {
                Cow::Borrowed(s) => Some(&s[i..j]),
                Cow::Owned(_) => None,
            },
            CookieStr::Concrete(_) => None,
        }
    }

    fn into_owned(self) -> CookieStr<'static> {
        match self {
            CookieStr::Indexed(a, b) => CookieStr::Indexed(a, b),
            CookieStr::Concrete(Cow::Owned(c)) => CookieStr::Concrete(Cow::Owned(c)),
            CookieStr::Concrete(Cow::Borrowed(c)) => CookieStr::Concrete(Cow::Owned(c.into())),
        }
    }
}

#[derive(Debug)]
pub struct Cookie<'a> {
    pub(crate) cookie_string: Option<Cow<'a, str>>,
    pub(crate) name: CookieStr<'a>,
    pub(crate) val: CookieStr<'a>,
    pub(crate) expires: Option<Expiration>,
    pub(crate) max_age: Option<Duration>,
    pub(crate) domain: Option<CookieStr<'a>>,
    pub(crate) path: Option<CookieStr<'a>>,
    pub(crate) secure: Option<bool>,
    pub(crate) http_only: Option<bool>,
    pub(crate) same_site: Option<SameSite>,
}

impl<'a> Cookie<'a> {
    pub fn parse(str: &'a str) -> Result<Cookie<'a>, ParseError> {
        parse_cookie(str)
    }

    pub fn builder(name: &'a str, val: &'a str) -> CookieBuilder<'a> {
        CookieBuilder::new(name, val)
    }

    pub fn name(&self) -> &str {
        self.name.as_str(self.cookie_string.as_ref())
    }

    pub fn name_raw(&self) -> Option<&str> {
        self.cookie_string
            .as_ref()
            .and_then(|s| self.name.to_raw_str(s))
    }

    pub fn value(&self) -> &str {
        self.val.as_str(self.cookie_string.as_ref())
    }

    pub fn value_raw(&self) -> Option<&str> {
        self.cookie_string
            .as_ref()
            .and_then(|s| self.val.to_raw_str(s))
    }

    pub fn name_value(&self) -> (&str, &str) {
        (self.name(), self.value())
    }

    pub fn expires(&self) -> Option<Expiration> {
        self.expires
    }

    pub fn max_age(&self) -> Option<Duration> {
        self.max_age
    }

    pub fn domain(&self) -> Option<&str> {
        match &self.domain {
            Some(domain) => {
                let domain = domain.as_str(self.cookie_string.as_ref());
                domain.strip_prefix(".").or(Some(domain))
            }
            None => None,
        }
    }

    pub fn domain_raw(&self) -> Option<&str> {
        match (self.domain.as_ref(), self.cookie_string.as_ref()) {
            (Some(domain), Some(source)) => match domain.to_raw_str(source) {
                Some(domain) => domain.strip_prefix(".").or(Some(domain)),
                None => None,
            },
            _ => None,
        }
    }

    pub fn path(&self) -> Option<&str> {
        match &self.path {
            Some(path) => Some(path.as_str(self.cookie_string.as_ref())),
            None => None,
        }
    }

    pub fn path_raw(&self) -> Option<&str> {
        match (self.path.as_ref(), self.cookie_string.as_ref()) {
            (Some(path), Some(source)) => path.to_raw_str(source),
            _ => None,
        }
    }

    pub fn secure(&self) -> Option<bool> {
        self.secure
    }

    pub fn http_only(&self) -> Option<bool> {
        self.http_only
    }

    pub fn same_site(&self) -> Option<SameSite> {
        self.same_site
    }

    pub fn set_expires(&mut self, val: Expiration) -> &mut Self {
        self.expires = Some(val);
        self
    }
    pub fn set_max_age(&mut self, val: Duration) -> &mut Self {
        self.max_age = Some(val);
        self
    }
    pub fn set_domain(&mut self, val: &'a str) -> &mut Self {
        self.domain = Some(CookieStr::Concrete(val.into()));
        self
    }
    pub fn set_path(&mut self, val: &'a str) -> &mut Self {
        self.path = Some(CookieStr::Concrete(val.into()));
        self
    }
    pub fn set_secure(&mut self, val: bool) -> &mut Self {
        self.secure = Some(val);
        self
    }
    pub fn set_http_only(&mut self, val: bool) -> &mut Self {
        self.http_only = Some(val);
        self
    }
    pub fn set_same_site(&mut self, val: SameSite) -> &mut Self {
        self.same_site = Some(val);
        self
    }

    pub fn into_owned(self) -> Cookie<'static> {
        Cookie {
            cookie_string: self.cookie_string.map(|s| s.into_owned().into()),
            name: self.name.into_owned(),
            val: self.val.into_owned(),
            expires: self.expires,
            max_age: self.max_age,
            domain: self.domain.map(|s| s.into_owned().into()),
            path: self.path.map(|s| s.into_owned().into()),
            secure: self.secure,
            http_only: self.http_only,
            same_site: self.same_site,
        }
    }
}

impl<'a> Display for Cookie<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}={}",
            self.name.as_str(self.cookie_string.as_ref()),
            self.val.as_str(self.cookie_string.as_ref())
        )?;

        if self.expires.is_some() {
            match self.expires.as_ref().unwrap() {
                Expiration::DateTime(date) => {
                    write!(f, "; Expires={} GMT", date.format("%a, %d %b %Y %H:%M:%S"))?;
                }
                Expiration::Session => {}
            }
        }
        if let Some(max_age) = self.max_age {
            write!(f, "; Max-Age={}", max_age.as_secs())?;
        }
        if let Some(domain) = self.domain.as_ref() {
            write!(f, "; Domain={}", domain.as_str(self.cookie_string.as_ref()))?;
        }
        if let Some(path) = self.path.as_ref() {
            write!(f, "; Path={}", path.as_str(self.cookie_string.as_ref()))?;
        }
        if let Some(true) = self.secure {
            write!(f, "; Secure")?;
        }
        if let Some(true) = self.http_only {
            write!(f, "; HttpOnly")?;
        }
        if let Some(same_site) = self.same_site.as_ref() {
            write!(f, "; SameSite={:?}", same_site)?;
        }

        Ok(())
    }
}
