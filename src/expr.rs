use chumsky::span::SimpleSpan;

use crate::parser::{Block, Ident};

pub type Span = SimpleSpan;

#[derive(Debug)]
pub enum Expr<'a> {
    // TODO: use i53
    // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/MAX_SAFE_INTEGER
    Int(i32),
    Parens(Box<Expr<'a>>),
    Var {
        name: Ident<'a>,
        name_span: Span,
    },

    Neg(Box<Expr<'a>>),
    Add(Box<Expr<'a>>, Box<Expr<'a>>),
    Sub(Box<Expr<'a>>, Box<Expr<'a>>),
    Mul(Box<Expr<'a>>, Box<Expr<'a>>),
    Div(Box<Expr<'a>>, Box<Expr<'a>>),

    Call(String, Vec<Expr<'a>>),
    Fn {
        name: &'a str,
        args: Vec<String>,
        body: Box<Expr<'a>>,
        then: Box<Expr<'a>>,
    },
    Block(Block<'a>),
}

impl Expr<'_> {
    pub fn contains_block(&self) -> bool {
        match self {
            Expr::Int(_) => false,
            Expr::Parens(expr) => expr.contains_block(),
            Expr::Var {
                name: _,
                name_span: _,
            } => false,
            Expr::Neg(expr) => expr.contains_block(),
            Expr::Add(lhs, rhs) => lhs.contains_block() || rhs.contains_block(),
            Expr::Sub(lhs, rhs) => lhs.contains_block() || rhs.contains_block(),
            Expr::Mul(lhs, rhs) => lhs.contains_block() || rhs.contains_block(),
            Expr::Div(lhs, rhs) => lhs.contains_block() || rhs.contains_block(),
            Expr::Call(_, args) => args.iter().any(Expr::contains_block),
            Expr::Fn { .. } => todo!("Move FN decls from Expr to Stmt"),
            Expr::Block(_) => true,
        }
    }

    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}
