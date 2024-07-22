use crate::syntax::{Expr, Ident};

#[derive(Debug)]
pub struct Let<'a> {
    pub name: Ident<'a>,
    pub ty: Option<Ident<'a>>,
    pub rhs: Expr<'a>,
}
