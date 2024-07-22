use crate::syntax::Item;

#[derive(Debug)]
pub struct File<'a> {
    pub items: Vec<Item<'a>>,
}
