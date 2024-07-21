use chumsky::container::Container;
use chumsky::container::OrderedSeq;
use chumsky::extra::Err;
use chumsky::prelude::*;
use chumsky::Parser;

use crate::expr::Expr;

type Extra<'a> = Err<Rich<'a, char>>;

pub fn parser<'a>() -> impl Parser<'a, &'a str, Expr<'a>, Extra<'a>> {
    block(expr()).then_ignore(end())
}

fn block<'a>(
    expr: impl Parser<'a, &'a str, Expr<'a>, Extra<'a>> + Clone,
) -> impl Parser<'a, &'a str, Expr<'a>, Extra<'a>> + Clone {
    let stmts = stmt(expr)
        .repeated()
        .collect::<Vec<_>>()
        .map(|stmts| Expr::Block { stmts });

    lbrace().ignore_then(stmts).then_ignore(rbrace())
}

fn token<'a, T>(c: T) -> impl Parser<'a, &'a str, (), Extra<'a>> + Clone
where
    T: OrderedSeq<'a, char> + Clone,
{
    just(c).padded().ignored()
}

fn lbrace<'a>() -> impl Parser<'a, &'a str, (), Extra<'a>> + Clone {
    token('{')
}

fn rbrace<'a>() -> impl Parser<'a, &'a str, (), Extra<'a>> + Clone {
    token('}')
}

fn semi<'a>() -> impl Parser<'a, &'a str, (), Extra<'a>> + Clone {
    token(';')
}

fn colon<'a>() -> impl Parser<'a, &'a str, (), Extra<'a>> + Clone {
    token(':')
}

fn equals<'a>() -> impl Parser<'a, &'a str, (), Extra<'a>> + Clone {
    token('=')
}

fn stmt<'a>(
    expr: impl Parser<'a, &'a str, Expr<'a>, Extra<'a>> + Clone,
) -> impl Parser<'a, &'a str, Expr<'a>, Extra<'a>> + Clone {
    // TODO: pull into individual stmts, so that blocks don't need to be ; terminated
    let end_of_statement = semi().or(rbrace().rewind());

    choice((stmt_let(expr.clone()), expr)).then_ignore(end_of_statement)
}

fn stmt_let<'a>(
    expr: impl Parser<'a, &'a str, Expr<'a>, Extra<'a>> + Clone,
) -> impl Parser<'a, &'a str, Expr<'a>, Extra<'a>> + Clone {
    text::ascii::keyword("let")
        .ignore_then(ident())
        .then(colon().ignore_then(ident()).or_not())
        .then_ignore(equals())
        .then(expr)
        .map(|((name, ty), rhs)| Expr::Let {
            name,
            ty,
            rhs: rhs.boxed(),
        })
        .padded()
}

fn expr<'a>() -> impl Parser<'a, &'a str, Expr<'a>, Extra<'a>> + Clone {
    recursive(|expr| {
        let block_expr = block(expr.clone());

        let parens_expr = expr
            .delimited_by(just('('), just(')'))
            .map(|expr: Expr| Expr::Parens(expr.boxed()))
            .padded();

        let var = ident().map_with(|name, extra| Expr::Var {
            name,
            name_span: extra.span(),
        });

        let atom = choice((block_expr, int(), parens_expr, var));

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
