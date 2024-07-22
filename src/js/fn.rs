use std::fmt::{self, Debug};

use crate::js::{Block, Display};

#[derive(Debug)]
pub struct Fn<'a> {
    pub name: &'a str,
    pub args: Vec<&'a str>,
    pub body: Block<'a>,
}

impl fmt::Display for Display<'_, Fn<'_>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_indent(f)?;

        write!(f, "function {}(", self.value.name)?;

        for (i, arg) in self.value.args.iter().enumerate() {
            if i + 1 == self.value.args.len() {
                write!(f, "{}", arg)?;
            } else {
                write!(f, "{}, ", arg)?;
            }
        }

        writeln!(f, ")")?;

        self.with(&self.value.body).fmt(f)?;

        Ok(())
    }
}
