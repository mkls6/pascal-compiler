use crate::io::CharReader;

#[allow(dead_code)]
pub struct Lexer<'a> {
    chars: CharReader<'a>,
}
