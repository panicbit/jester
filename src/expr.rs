use chumsky::span::SimpleSpan;

pub type Span = SimpleSpan;

#[derive(Debug)]
pub enum Expr<'a> {
    // TODO: use i53
    // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/MAX_SAFE_INTEGER
    Int(i32),
    Parens(Box<Expr<'a>>),
    Var {
        name: &'a str,
        name_span: Span,
    },

    Neg(Box<Expr<'a>>),
    Add(Box<Expr<'a>>, Box<Expr<'a>>),
    Sub(Box<Expr<'a>>, Box<Expr<'a>>),
    Mul(Box<Expr<'a>>, Box<Expr<'a>>),
    Div(Box<Expr<'a>>, Box<Expr<'a>>),

    Call(String, Vec<Expr<'a>>),
    Let {
        name: &'a str,
        ty: Option<&'a str>,
        rhs: Box<Expr<'a>>,
        then: Box<Expr<'a>>,
    },
    Fn {
        name: &'a str,
        args: Vec<String>,
        body: Box<Expr<'a>>,
        then: Box<Expr<'a>>,
    },
}

impl Expr<'_> {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}
