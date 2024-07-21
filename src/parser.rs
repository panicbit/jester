use chumsky::extra::Err;
use chumsky::prelude::*;
use chumsky::Parser;

use crate::expr::Expr;

type Extra<'a> = Err<Rich<'a, char>>;

pub fn parser<'a>() -> impl Parser<'a, &'a str, Expr<'a>, Extra<'a>> {
    block().then_ignore(end())
}

fn block<'a>() -> impl Parser<'a, &'a str, Expr<'a>, Extra<'a>> + Clone {
    just('{').ignore_then(decl()).then_ignore(just('}'))
}

fn decl<'a>() -> impl Parser<'a, &'a str, Expr<'a>, Extra<'a>> + Clone {
    recursive(|decl| {
        let r#let = text::ascii::keyword("let")
            .ignore_then(ident())
            .then(just(':').ignore_then(ident()).or_not())
            .then_ignore(just('='))
            .then(expr())
            .then_ignore(just(';'))
            .then(decl.clone())
            .map(|(((name, ty), rhs), then)| Expr::Let {
                name,
                ty,
                rhs: rhs.boxed(),
                then: Box::new(then),
            });

        r#let
            // Must be later in the chain than `r#let` to avoid ambiguity
            .or(expr())
            .padded()
    })
}

fn expr<'a>() -> impl Parser<'a, &'a str, Expr<'a>, Extra<'a>> + Clone {
    recursive(|expr| {
        let parens_expr = expr
            .delimited_by(just('('), just(')'))
            .map(|expr: Expr| Expr::Parens(expr.boxed()))
            .padded();

        let var = ident().map_with(|name, extra| Expr::Var {
            name,
            name_span: extra.span(),
        });

        let atom = choice((int(), parens_expr, var));

        let op = |c| just(c).padded();

        let unary = op('-')
            .repeated()
            .foldr(atom, |_op, rhs| Expr::Neg(rhs.boxed()));

        let product = unary.clone().foldl(
            choice((
                op('*').to(Expr::Mul as fn(_, _) -> _),
                op('/').to(Expr::Div as fn(_, _) -> _),
            ))
            .then(unary)
            .repeated(),
            |lhs, (op, rhs)| op(lhs.boxed(), rhs.boxed()),
        );

        let sum = product.clone().foldl(
            choice((
                op('+').to(Expr::Add as fn(_, _) -> _),
                op('-').to(Expr::Sub as fn(_, _) -> _),
            ))
            .then(product)
            .repeated(),
            |lhs, (op, rhs)| op(lhs.boxed(), rhs.boxed()),
        );

        sum
    })
}

fn ident<'a>() -> impl Parser<'a, &'a str, &'a str, Extra<'a>> + Clone {
    text::ascii::ident().padded()
}

fn int<'a>() -> impl Parser<'a, &'a str, Expr<'a>, Extra<'a>> + Clone {
    text::int(10)
        .map(|s: &str| Expr::Int(s.parse().unwrap()))
        .padded()
}
