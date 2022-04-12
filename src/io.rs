use std::iter::Peekable;
use std::str::Chars;

pub struct CharReader<'a> {
    current_char: Option<char>,
    chars: Peekable<Chars<'a>>,
}

impl<'a> CharReader<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.chars().peekable();
        let current_char = chars.next();

        Self {
            current_char,
            chars,
        }
    }

    #[allow(dead_code)]
    pub fn current_char(&self) -> Option<char> {
        self.current_char
    }

    #[allow(dead_code)]
    pub fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }
}

impl<'a> Iterator for CharReader<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_char = self.chars.next();
        self.current_char
    }
}
