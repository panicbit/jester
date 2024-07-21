use std::borrow::Cow;
use std::fmt;

#[derive(Debug)]
pub enum Expr<'a> {
    Number(f64),
    Let {
        name: Cow<'a, str>,
        rhs: Box<Expr<'a>>,
    },
    Parens(Box<Expr<'a>>),
    Var(Cow<'a, str>),
    Neg(Box<Expr<'a>>),
    Mul(Box<Expr<'a>>, Box<Expr<'a>>),
    Div(Box<Expr<'a>>, Box<Expr<'a>>),
    Add(Box<Expr<'a>>, Box<Expr<'a>>),
    Sub(Box<Expr<'a>>, Box<Expr<'a>>),
}

impl Expr<'_> {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl fmt::Display for Expr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let expr = DisplayJSExpr {
            indent: 0,
            expr: self,
        };

        write!(f, "{expr}")
    }
}

struct DisplayJSExpr<'a> {
    indent: usize,
    expr: &'a Expr<'a>,
}

impl<'a> DisplayJSExpr<'a> {
    pub fn write_indent(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.indent {
            write!(f, "    ")?;
        }

        Ok(())
    }

    pub fn with(&self, expr: &'a Expr<'a>) -> Self {
        Self {
            indent: self.indent,
            expr,
        }
    }
}

impl<'a> fmt::Display for DisplayJSExpr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_indent(f)?;

        match &self.expr {
            Expr::Number(n) => write!(f, "{n}"),
            Expr::Let { name, rhs } => writeln!(f, "let {name} = {rhs};"),
            Expr::Parens(expr) => write!(f, "({expr})"),
            Expr::Var(name) => write!(f, "{name}"),
            Expr::Neg(expr) => match **expr {
                Expr::Neg(_) => write!(f, "-({expr})"),
                _ => write!(f, "-{expr}"),
            },
            Expr::Mul(lhs, rhs) => write!(f, "{lhs} * {rhs}"),
            Expr::Div(lhs, rhs) => write!(f, "{lhs} / {rhs}"),
            Expr::Add(lhs, rhs) => write!(f, "{lhs} + {rhs}"),
            Expr::Sub(lhs, rhs) => write!(f, "{lhs} - {rhs}"),
        }
    }
}
