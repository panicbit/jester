use std::fmt;

use crate::js::{self, Stmt};

#[derive(Debug)]
pub struct Block<'a> {
    pub stmts: Vec<Stmt<'a>>,
}

impl<'a> fmt::Display for js::Display<'a, Block<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_indent(f)?;
        writeln!(f, "{{")?;

        for stmt in &self.value.stmts {
            self.with_indented(stmt).fmt(f)?;
        }

        self.write_indent(f)?;
        writeln!(f, "}}")?;

        Ok(())
    }
}
