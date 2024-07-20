use chumsky::extra::Err;
use chumsky::prelude::*;
use chumsky::Parser;

use crate::expr::Expr;

pub fn parser<'a>() -> impl Parser<'a, &'a str, Expr<'a>, Err<Rich<'a, char>>> {
    let ident = text::ascii::ident().padded();

    let expr = recursive(|expr| {
        let int = text::int(10)
            .map(|s: &str| Expr::Int(s.parse().unwrap()))
            .padded();

        let parens_expr = expr
            .delimited_by(just('('), just(')'))
            .map(|expr: Expr| Expr::Parens(expr.boxed()))
            .padded();

        let var = ident.map_with(|name, extra| Expr::Var(name, extra.span()));

        let atom = choice((int, parens_expr, var));

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
    });

    let decl = recursive(|decl| {
        let r#let = text::ascii::keyword("let")
            .ignore_then(ident)
            .then_ignore(just('='))
            .then(expr.clone())
            .then_ignore(just(';'))
            .then(decl.clone())
            .map(|((name, rhs), then)| Expr::Let {
                name,
                rhs: rhs.boxed(),
                then: Box::new(then),
            });

        r#let
            // Must be later in the chain than `r#let` to avoid ambiguity
            .or(expr)
            .padded()
    });

    decl.then_ignore(end())
}
