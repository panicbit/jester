use chumsky::container::OrderedSeq;
use chumsky::extra::Err;
use chumsky::extra::Full;
use chumsky::prelude::*;
use chumsky::Parser as _;

use crate::syntax::r#fn;
use crate::syntax::Block;
use crate::syntax::Expr;
use crate::syntax::File;
use crate::syntax::Fn;
use crate::syntax::Ident;
use crate::syntax::Item;
use crate::syntax::Let;
use crate::syntax::Stmt;

type Extra<'a> = Full<Error<'a>, State, Context>;
type Error<'a> = Rich<'a, char>;
type State = ();
type Context = ();

pub trait Parser<'a, O>: chumsky::Parser<'a, &'a str, O, Extra<'a>> + Clone {}

impl<'a, O, T> Parser<'a, O> for T where T: chumsky::Parser<'a, &'a str, O, Extra<'a>> + Clone {}

pub fn file<'a>() -> impl Parser<'a, File<'a>> {
    item()
        .repeated()
        .collect::<Vec<_>>()
        .then_ignore(end())
        .map(|items| File { items })
}

fn item<'a>() -> impl Parser<'a, Item<'a>> {
    r#fn().map(Item::Fn)
}

fn r#fn<'a>() -> impl Parser<'a, Fn<'a>> {
    let name = ident();
    let arg = ident()
        .then_ignore(token(':'))
        .then(r#type())
        .map(|(name, r#type)| r#fn::Arg { name, r#type });
    let args = arg
        .separated_by(token(','))
        .allow_trailing()
        .collect::<Vec<_>>();
    let return_value = token("->").ignore_then(r#type()).or_not().labelled("->");
    let body = block(expr());

    token("fn")
        .labelled("fn")
        .ignore_then(name)
        .then(args.delimited_by(token('('), token(')')))
        .then(return_value)
        .then(body)
        .map(|(((name, args), return_type), body)| Fn {
            name,
            args,
            return_type,
            body,
        })
}

fn block<'a>(expr: impl Parser<'a, Expr<'a>>) -> impl Parser<'a, Block<'a>> {
    let stmts = stmt(expr).repeated().collect::<Vec<_>>();

    lbrace()
        .ignore_then(stmts)
        .then_ignore(rbrace())
        .map(|stmts| Block { stmts })
}

fn token<'a, T>(c: T) -> impl Parser<'a, ()>
where
    T: OrderedSeq<'a, char> + Clone,
{
    just(c).padded().ignored()
}

fn lbrace<'a>() -> impl Parser<'a, ()> {
    token('{')
}

fn rbrace<'a>() -> impl Parser<'a, ()> {
    token('}')
}

fn semi<'a>() -> impl Parser<'a, ()> {
    token(';')
}

fn colon<'a>() -> impl Parser<'a, ()> {
    token(':')
}

fn equals<'a>() -> impl Parser<'a, ()> {
    token('=')
}

fn stmt<'a>(expr: impl Parser<'a, Expr<'a>>) -> impl Parser<'a, Stmt<'a>> {
    // TODO: pull into individual stmts, so that blocks don't need to be ; terminated
    let end_of_statement = semi().or(rbrace().rewind());
    let expr_stmt = expr.clone().map(Stmt::Expr);

    choice((stmt_let(expr), expr_stmt)).then_ignore(end_of_statement)
}

fn stmt_let<'a>(expr: impl Parser<'a, Expr<'a>>) -> impl Parser<'a, Stmt<'a>> {
    text::ascii::keyword("let")
        .ignore_then(ident())
        .then(colon().ignore_then(ident()).or_not())
        .then_ignore(equals())
        .then(expr)
        .map(|((name, ty), rhs)| Stmt::Let(Let { name, ty, rhs }))
        .padded()
}

fn expr<'a>() -> impl Parser<'a, Expr<'a>> {
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

fn ident<'a>() -> impl Parser<'a, Ident<'a>> {
    text::ascii::ident()
        .padded()
        .map(Ident::new)
        .labelled("identifier")
}

fn r#type<'a>() -> impl Parser<'a, Ident<'a>> {
    ident().labelled("type")
}

fn int<'a>() -> impl Parser<'a, Expr<'a>> {
    text::int(10)
        .map(|s: &str| Expr::Int(s.parse().unwrap()))
        .padded()
}
