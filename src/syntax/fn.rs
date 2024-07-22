use crate::syntax::{Block, Ident};

#[derive(Debug)]
pub struct Fn<'a> {
    pub name: Ident<'a>,
    pub args: Vec<Arg<'a>>,
    pub return_type: Option<Ident<'a>>,
    pub body: Block<'a>,
}

#[derive(Debug)]
pub struct Arg<'a> {
    pub name: Ident<'a>,
    pub r#type: Ident<'a>,
}
