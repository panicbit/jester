use std::fmt;

use crate::js::{Display, Stmt};

#[derive(Debug)]
pub struct File<'a> {
    pub stmts: Vec<Stmt<'a>>,
}

impl fmt::Display for Display<'_, File<'_>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.value.stmts {
            self.with(item).fmt(f)?;
        }

        Ok(())
    }
}
