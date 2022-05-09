use std::fs::File;
use std::io::{BufRead, BufReader, Error, Lines};

pub struct CharReader {
    current_char: Option<char>,
    chars: Option<Vec<char>>,
    lines: Lines<BufReader<File>>,
    line_num: usize,
    col_num: usize,
}

impl CharReader {
    pub fn new(filename: String) -> Result<Self, Error> {
        let file = File::open(filename)?;

        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let chars: Option<Vec<char>> = match lines.by_ref().next() {
            Some(Ok(s)) => {
                let mut c: Vec<char> = s.chars().collect();
                c.push('\n');
                Some(c)
            }
            _ => None,
        };

        let line_num = 1;
        let col_num = 0;

        let current_char = match chars.as_ref() {
            Some(v) => Some(v[0]),
            _ => None,
        };

        let reader = Self {
            current_char,
            chars,
            lines,
            line_num,
            col_num,
        };
        Ok(reader)
    }

    pub fn current_char(&self) -> Option<char> {
        self.current_char
    }

    pub fn peek(&mut self) -> Option<&char> {
        match self.chars.as_ref() {
            Some(v) if v.len() == self.col_num + 1 => None,
            Some(v) => Some(&v[self.col_num + 1]),
            _ => None,
        }
    }

    pub fn position(&self) -> (usize, usize) {
        (self.line_num, self.col_num + 1)
    }
}

impl Iterator for CharReader {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.chars.as_ref() {
            // End of current line => we need to pass \n and read next line
            Some(v) if self.col_num + 1 == v.len() => {
                self.line_num += 1;
                self.col_num = 0;

                // Loop until non-empty line or EOF
                loop {
                    match self.lines.by_ref().next() {
                        Some(Ok(s)) if s.len() > 0 => {
                            let mut c: Vec<char> = s.chars().collect();
                            c.push('\n');

                            self.current_char = Some(c[0]);
                            self.chars = Some(c);
                            break;
                        }
                        None => {
                            self.chars = None;
                            self.current_char = None;
                            break;
                        }
                        _ => (),
                    };
                }
            }
            Some(v) => {
                self.col_num += 1;
                self.current_char = Some(v[self.col_num]);
            }
            _ => {
                self.current_char = None;
            }
        };

        self.current_char
    }
}
