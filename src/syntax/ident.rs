use std::ops;

#[derive(Debug)]
pub struct Ident<'a>(&'a str);

impl<'a> Ident<'a> {
    pub fn new(ident: &'a str) -> Self {
        Self(ident)
    }

    pub fn as_str(&self) -> &'a str {
        self.0
    }
}

impl<'a> ops::Deref for Ident<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}
