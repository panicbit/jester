use std::borrow::Cow;
use std::fmt;

#[derive(Debug)]
pub enum Stmt<'a> {
    Let(Let<'a>),
    Expr(Expr<'a>),
    Block(Block<'a>),
}

impl fmt::Display for Stmt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DisplayJS::new(self).fmt(f)
    }
}

impl<'a> fmt::Display for DisplayJS<'a, Stmt<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            Stmt::Let(let_stmt) => self.with(let_stmt).fmt(f),
            Stmt::Expr(expr) => writeln!(f, "{}", self.with(expr)),
            Stmt::Block(block) => self.with(block).fmt(f),
        }
    }
}

#[derive(Debug)]
pub struct Let<'a> {
    pub name: Cow<'a, str>,
    pub rhs: Box<Expr<'a>>,
}

impl<'a> fmt::Display for DisplayJS<'a, Let<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_indent(f)?;

        let Let { name, rhs } = self.value;

        writeln!(f, "let {name} = {rhs};")
    }
}

#[derive(Debug)]
pub enum Expr<'a> {
    Number(f64),
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
        DisplayJS::new(self).fmt(f)
    }
}

pub struct DisplayJS<'a, T: 'a> {
    indent: usize,
    value: &'a T,
}

impl<'a, T: 'a> DisplayJS<'a, T> {
    pub fn new(value: &'a T) -> Self {
        Self { indent: 0, value }
    }

    pub fn write_indent(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.indent {
            write!(f, "    ")?;
        }

        Ok(())
    }

    pub fn with<'b, U: 'b>(&self, value: &'b U) -> DisplayJS<'b, U> {
        DisplayJS {
            indent: self.indent,
            value,
        }
    }

    pub fn with_indented<'b, U: 'b>(&self, value: &'b U) -> DisplayJS<'b, U> {
        DisplayJS {
            indent: self.indent + 1,
            value,
        }
    }
}

impl<'a> fmt::Display for DisplayJS<'a, Expr<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_indent(f)?;

        match &self.value {
            Expr::Number(n) => write!(f, "{n}"),
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

#[derive(Debug)]
pub struct Block<'a> {
    pub stmts: Vec<Stmt<'a>>,
}

impl<'a> fmt::Display for DisplayJS<'a, Block<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_indent(f)?;
        writeln!(f, "{{");

        for stmt in &self.value.stmts {
            self.with_indented(stmt).fmt(f)?;
        }

        self.write_indent(f)?;
        writeln!(f, "}}")?;

        Ok(())
    }
}
