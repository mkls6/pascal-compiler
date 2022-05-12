# Начало написания компилятора. Лексический анализ.
Компилятор - это компьютерная программа, транслирующая исходный код на одном формальном языке в другой,
чаще всего в машинный код для конкретной аппаратной платформы.

Если представлять компилятор как чёрный ящик, то получится следующее:
![компилятор чёрный ящик](imgs/compiler_black_box.png)

Принципы компиляции:
1. **Компилятор должен сохранять смысл компилируемой программы.**

Нет смысла пользоваться компилятором, который для корреткного исходного кода
генерирует некорректную программу. Сгенерированная программа должна делать
только то, что было написано на исходном языке;

2. **Компилятор должен улучшать входную программу некоторым заметным образом.**

Под улучшением в данном случае понимается как сама возможность выполнения на
целевом компьютере, так и внесение таких изменений, которые меняют исходный код
для достижения некоторого более оптимального представления (это может быть как
ускорение времени выполнения путём, например, поисков инвариантов в циклах,
удаления неиспользуемого кода, так и достижение более компактного представления,
занимающего меньше места на носителе).


## Структура компилятора
Перед тем, как начать реализацию, необходимо чётко разделить
весь компилятор на отдельные части в зависимости от назначения. Чтобы это сделать,
необходимо понять, на какие этапы делится сам процесс компиляции
(на текущий момент без углубления в детали).

Можно выделить следующие этапы:
1. Лексический анализ

Задача лексического анализа - разбить исходный текст на набор лексем (токенов) с построением
внутреннего представления.
Поток лексем затем передаётся на вход следующему компоненту - синтаксическому анализатору.
Компонент, который выполняет лексический анализ часто называется **сканнером** или **лексёром**.

2. Синтаксический анализ

Задача синтаксического анализа - сопоставление последовательности лексем с формальной
грамматикой языка.

3. Семантический анализ

Задача семантического анализа - проверить соответствие исходного текста
неформальным правилам языка. Иными словами, исходный текст должен
быть осмысленным.

4. Генерация

На этом этапе происходит генерация кода для целевой платформы/целевого языка.

Для каждого из этапов можно выделить отдельный компонент компилятора, а также
компонент ввода-вывода, постепенно считывающий исходный текст и передающий его на
вход анализатору.

Этапы могут выполняться последовательно или параллельно (с синхронизацией при необходимости).

**Проход** - это чтение исходного текста программы.

**Однопроходные** компиляторы просматривают весь исходный текст ровно один раз.
Ограничение - все идентификаторы должны быть описаны до использования.

## Проектирование модуля ввода/вывода
Первое, с чего начинается работа любого компилятора - это считывание исходного текста
из некоторого источника (стандартный ввод, файл). Для этого реализуем отдельный компонент.

Составим требования к модулю ввода/вывода:
1. Считывание исходного кода происходит из **файла**
2. Считывание происходит постепенно (**построчно**)
3. Модуль ввода/вывода должен предоставлять интерфейс для **посимвольной** итерации
4. Должна присутствовать возможность заглянуть на несколько символов вперёд (в рамках одной строки).

## Реализация модуля ввода/вывода
Полная реализация расположена в файле `src/io.rs`.

Компонент реализован в виде структуры `CharReader` со следующими полями:
```rust
pub struct CharReader {
    current_char: Option<char>,
    chars: Option<Vec<char>>,
    lines: Lines<BufReader<File>>,
    line_num: usize,
    col_num: usize,
}
```
- `current_char` - текущий (последний) просматриваемый символ
- `chars` - символы текущей (последней прочитанной) строки
- `lines` - итератор построчного буферизированного чтения из файла
- `line_num` - номер текущей просматриваемой строки
- `col_num` - номер текущего просматриваемого столбца (символа) в строке

`current_char` и `chars` имеют тип `Option<T>`. Когда файл прочитан полностью (или он изначально пуст),
принимают значение `None`, иначе `Some(T)`.

Конструктор (метод `new()`) принимает один аргумент - путь до сканируемого файла.
`new()` возвращает `Ok(CharReader)`, если файл существует, и `Err(std::io::Error)`, если при открытии произошла ошибка.

```rust
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
```

Для посимвольного обхода реализована черта (trait) `Iterator`:
```rust
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
```

Для этого реализован метод `next()`, который, согласно интерфейсу, возвращает `Option<Item>`.
`None` - сигнал исчерпания потока. В ином случае возвращается `Some(char)`.
На каждом вызове метода мы передвигаем "курсор" на одну позицию, запоминая новый символ.
Если был достигнут конец строки, происходит переход на следующую, если она есть.

## Проектирование лексического анализатора
Лексический анализатор должен обрабатывать последовательность символов,
выделяя из неё лексемы (токены) и обрабатывая лексические ошибки.

Лексический анализатор должен предоставлять интерфейс для обхода
выделенных токенов, как последовательности, то есть реализовывать
интерфейс итератора.

## Реализация лексического анализатора
Для удобства все виды токенов представлены в виде перечислимого
типа данных:
```rust

#[derive(Debug)]
pub enum Token {
    Integer(i32),
    Identifier(String),
    StringLiteral(String),
    IntegerKeyword,
    RealKeyword,
    Real(f32),
    ProgramKeyword,
    VarKeyword,
    BeginKeyword,
    EndKeyword,
    PlusOp,
    MinusOp,
    MulOp,
    DivOp,
    ModOp,
    AssignOp,
    Colon,
    Period,
    LBrace,
    RBrace,
    Semicolon,
    EOF,
}
```
Часть элементов перечислимого типа имеют кортежную структуру -
аргумент, используемый для создания. Например, числовые или строковые
литералы. Для внутреннего использования также введён дополнительный токен
EOF - символ конца потока, используемый во внутренней логике работы
анализатора.

Для представления лексических ошибок создана отдельная структура `LexicalError`:
```rust
pub struct LexicalError {
    description: String,
    line: usize,
    column: usize,
}

impl LexicalError {
    pub fn new(description: String, line: usize, column: usize) -> Self {
        Self {
            description,
            line,
            column,
        }
    }
}
```

Структура содержит строковое описание ошибки, а также позицию в файле, где она произошла.

Лексический анализатор задан в виде структуры:
```rust
pub struct Lexer {
    chars: CharReader,
}


impl Lexer {
    pub fn new(chars: CharReader) -> Self {
        Self { chars }
    }
    …
}

```

Для обхода токенов реализован trait `Iterator`:
```rust
impl Iterator for Lexer {
    type Item = Result<Token, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_ws();

        let token = match self.chars.by_ref().current_char() {
            Some(ch) => match ch {
                '0'..='9' => self.number(),
                '+' | '-' | '*' | ':' => self.operator(),
                _ if ch.is_alphanumeric() => self.maybe_keyword(),
                _ => self.symbol(),
            },
            None => Ok(Token::EOF),
        };

        match token {
            Ok(Token::EOF) => None,
            _ => Some(token),
        }
    }
}
```

В методе `next()` мы обходим посимвольно исходный файл при помощи
`CharReader`, где в зависимости от символа запускаем обработчик
конкретной лексемы.

Обработка чисел:
```rust
impl Lexer {
    …
    fn number(&mut self) -> Result<Token, LexicalError> {
        let mut num = String::new();
        let mut is_real = false;

        loop {
            match self.chars.by_ref().current_char() {
                Some(ch) if ch.is_digit(10) => num.push(ch),
                Some(ch) if ch == '.' => {
                    num.push(ch);
                    is_real = true;
                }
                Some(ch) if ch.is_whitespace() => break,
                Some(ch) if ch.is_alphanumeric() => {
                    // Consume everything until whitespace or EOF
                    num.push(ch);

                    while let Some(ch) = self.chars.next() {
                        if ch.is_whitespace() {
                            break;
                        } else {
                            num.push(ch);
                        }

                        break;
                    }
                }
                _ => break,
            }

            self.chars.by_ref().next();
        }

        if is_real {
            let parsed = num.parse::<f32>();

            match parsed {
                Ok(f) => Ok(Token::Real(f)),
                _ => {
                    let pos = self.chars.position();
                    Err(LexicalError::new(
                        String::from(format!("Invalid real literal {}", num)),
                        pos.0,
                        pos.1,
                    ))
                }
            }
        } else {
            let parsed = num.parse::<i32>();

            match parsed {
                Ok(i) => Ok(Token::Integer(i)),
                _ => {
                    let pos = self.chars.position();

                    Err(LexicalError::new(
                        String::from(format!("Invalid int literal {}", num)),
                        pos.0,
                        pos.1,
                    ))
                }
            }
        }
    }
    …
}
```
Обработка чисел запускается при выявлении цифры в начале очередной выделяемой лексемы.
Если в процессе была найдена точка, то пытаемся обработать последовательность
как вещественное число, иначе - как целочисленное. Если при обработке
возникла ошибка, то создаётся соответствующая лексическая ошибка и
передаётся как результат обработки. Если числовой литерал корректен,
то возвращается соответствующий токен.

Обработка операторов происходит в методе `operator()`:
```rust
fn operator(&mut self) -> Result<Token, LexicalError> {
    // Если поток символов иссяк
    let op = if self.chars.current_char().is_none() {
        Ok(Token::EOF)
    } else {
        match self.chars.current_char().unwrap() {
            '+' => Ok(Token::PlusOp),
            '-' => Ok(Token::MinusOp),
            '*' => Ok(Token::MulOp),
            ':' => match self.chars.by_ref().peek() {
                Some(ch) if ch == &'=' => {
                    self.chars.by_ref().next();
                    Ok(Token::AssignOp)
                }
                _ => Ok(Token::Colon),
            },
            _ => {
                let pos = self.chars.position();

                Err(LexicalError::new(
                    String::from("Invalid operator"),
                    pos.0,
                    pos.1,
                ))
            }
        }
    };

    self.chars.by_ref().next();
    op
}
```

Обработка специальных символов:
```rust
fn symbol(&mut self) -> Result<Token, LexicalError> {
    let sym = if self.chars.current_char().is_none() {
        Ok(Token::EOF)
    } else {
        match self.chars.current_char().unwrap() {
            ';' => Ok(Token::Semicolon),
            '.' => Ok(Token::Period),
            '(' => Ok(Token::LBrace),
            ')' => Ok(Token::RBrace),
            '\'' => {
                // Read chars until string literal is closed
                let literal: String = self
                    .chars
                    .by_ref()
                    .take_while(|x: &char| x != &'\'')
                    .collect();

                match self.chars.current_char() {
                    Some('\'') => Ok(Token::StringLiteral(literal)),
                    _ => {
                        let pos = self.chars.position();

                        Err(LexicalError::new(
                            String::from("Invalid string literal"),
                            pos.0,
                            pos.1,
                        ))
                    }
                }
            }
            _ => {
                let pos = self.chars.position();

                Err(LexicalError::new(
                    String::from(format!(
                        "Unsupported symbol {}",
                        self.chars.current_char().unwrap()
                    )),
                    pos.0,
                    pos.1,
                ))
            }
        }
    };

    self.chars.next();
    sym
}
```

Обработка идентификаторов и ключевых слов:
```rust
fn maybe_keyword(&mut self) -> Result<Token, LexicalError> {
    if self.chars.by_ref().current_char().is_none() {
        Ok(Token::EOF)
    } else {
        let mut s = String::new();
        s.push(self.chars.by_ref().current_char().unwrap());

        loop {
            match self.chars.next() {
                Some(ch) if ch.is_alphanumeric() => s.push(ch),
                _ => break,
            }
        }

        match s.to_lowercase().as_str() {
            "div" => Ok(Token::DivOp),
            "mod" => Ok(Token::ModOp),
            "program" => Ok(Token::ProgramKeyword),
            "begin" => Ok(Token::BeginKeyword),
            "end" => Ok(Token::EndKeyword),
            "integer" => Ok(Token::IntegerKeyword),
            "real" => Ok(Token::RealKeyword),
            "var" => Ok(Token::VarKeyword),
            _ => Ok(Token::Identifier(s)),
        }
    }
}
```