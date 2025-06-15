use std::time::Duration;

use crate::{
    cookie::{Cookie, CookieStr},
    expires::Expiration,
    same_site::SameSite,
};

pub struct CookieBuilder<'a>(Cookie<'a>);

impl<'a> CookieBuilder<'a> {
    pub fn new(name: &'a str, val: &'a str) -> Self {
        CookieBuilder(Cookie {
            cookie_string: None,
            name: CookieStr::Concrete(name.into()),
            val: CookieStr::Concrete(val.into()),
            expires: None,
            max_age: None,
            domain: None,
            path: None,
            secure: None,
            http_only: None,
            same_site: None,
        })
    }

    pub fn expires(mut self, expires: Expiration) -> Self {
        self.0.set_expires(expires);
        self
    }

    pub fn max_age(mut self, max_age: Duration) -> Self {
        self.0.set_max_age(max_age);
        self
    }

    pub fn domain(mut self, domain: &'a str) -> Self {
        self.0.set_domain(domain);
        self
    }

    pub fn path(mut self, path: &'a str) -> Self {
        self.0.set_path(path);
        self
    }

    pub fn secure(mut self, secure: bool) -> Self {
        self.0.set_secure(secure);
        self
    }

    pub fn http_only(mut self, http_only: bool) -> Self {
        self.0.set_http_only(http_only);
        self
    }

    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.0.set_same_site(same_site);
        self
    }

    pub fn build(self) -> Cookie<'a> {
        self.0
    }
}

impl<'a> From<Cookie<'a>> for CookieBuilder<'a> {
    fn from(value: Cookie<'a>) -> Self {
        CookieBuilder(value)
    }
}
