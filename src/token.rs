use std::any::type_name;
use std::error::Error;
use std::fmt;
use std::format;
use std::str::FromStr;

#[derive(Debug)]
pub struct Token<T> {
    value: T,
}

impl<T> Token<T> {
    pub fn new(value: T) -> Self {
        Token { value }
    }
}

impl<T> Token<T>
where
    T: FromStr + fmt::Display,
    <T as FromStr>::Err: fmt::Display,
{
    pub fn as_string(&self) -> String {
        String::from(format!("{}", self.value))
    }

    pub fn as_value(&self) -> &T {
        &self.value
    }

    pub fn from_string(string: String) -> Result<Self, Box<dyn Error>> {
        let value: T = match string.parse() {
            Ok(val) => val,
            Err(e) => panic!("{}", e),
        };

        Ok(Self { value })
    }
}

impl<T: fmt::Display> fmt::Display for Token<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Token of type {}, value = {}",
            type_name::<T>(),
            self.value
        )
    }
}

pub type Integer = Token<i32>;
pub type Real = Token<f32>;
pub type Boolean = Token<bool>;
pub type Str = Token<String>;
