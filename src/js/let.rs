use std::borrow::Cow;
use std::fmt;

use crate::js::{Display, Expr};

#[derive(Debug)]
pub struct Let<'a> {
    pub name: Cow<'a, str>,
    pub rhs: Box<Expr<'a>>,
}

impl<'a> fmt::Display for Display<'a, Let<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_indent(f)?;

        let Let { name, rhs } = self.value;

        writeln!(f, "let {name} = {rhs};")
    }
}
