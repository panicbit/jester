use crate::syntax::Fn;

#[derive(Debug)]
pub enum Item<'a> {
    Fn(Fn<'a>),
}
