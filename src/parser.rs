use chumsky::container::OrderedSeq;
use chumsky::extra::Err;
use chumsky::prelude::*;
use chumsky::Parser;

use crate::syntax::Block;
use crate::syntax::Expr;
use crate::syntax::Ident;
use crate::syntax::Let;
use crate::syntax::Stmt;

type Extra<'a> = Err<Rich<'a, char>>;

pub fn parser<'a>() -> impl Parser<'a, &'a str, Block<'a>, Extra<'a>> + Clone {
    block(expr()).then_ignore(end())
}

fn block<'a>(
    expr: impl Parser<'a, &'a str, Expr<'a>, Extra<'a>> + Clone,
) -> impl Parser<'a, &'a str, Block<'a>, Extra<'a>> + Clone {
    let stmts = stmt(expr).repeated().collect::<Vec<_>>();

    lbrace()
        .ignore_then(stmts)
        .then_ignore(rbrace())
        .map(|stmts| Block { stmts })
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
) -> impl Parser<'a, &'a str, Stmt<'a>, Extra<'a>> + Clone {
    // TODO: pull into individual stmts, so that blocks don't need to be ; terminated
    let end_of_statement = semi().or(rbrace().rewind());
    let expr_stmt = expr.clone().map(Stmt::Expr);

    choice((stmt_let(expr), expr_stmt)).then_ignore(end_of_statement)
}

fn stmt_let<'a>(
    expr: impl Parser<'a, &'a str, Expr<'a>, Extra<'a>> + Clone,
) -> impl Parser<'a, &'a str, Stmt<'a>, Extra<'a>> + Clone {
    text::ascii::keyword("let")
        .ignore_then(ident())
        .then(colon().ignore_then(ident()).or_not())
        .then_ignore(equals())
        .then(expr)
        .map(|((name, ty), rhs)| Stmt::Let(Let { name, ty, rhs }))
        .padded()
}

fn expr<'a>() -> impl Parser<'a, &'a str, Expr<'a>, Extra<'a>> + Clone {
    recursive(|expr| {
        let parenthized = expr
            .clone()
            .delimited_by(token('('), token(')'))
            .map(|expr: Expr| Expr::Parens(expr.boxed()))
            .padded();

        let var = ident().map_with(|name, extra| Expr::Var {
            name,
            name_span: extra.span(),
        });

        let block = block(expr).map(Expr::Block);

        let atom = choice((block, int(), parenthized, var));

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

fn ident<'a>() -> impl Parser<'a, &'a str, Ident<'a>, Extra<'a>> + Clone {
    text::ascii::ident().padded().map(Ident::new)
}

fn int<'a>() -> impl Parser<'a, &'a str, Expr<'a>, Extra<'a>> + Clone {
    text::int(10)
        .map(|s: &str| Expr::Int(s.parse().unwrap()))
        .padded()
}
