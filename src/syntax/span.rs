use chumsky::span::SimpleSpan;

pub type Span = SimpleSpan;

pub struct MySpan {
    start: usize,
    end: usize,
    path: String,
}
