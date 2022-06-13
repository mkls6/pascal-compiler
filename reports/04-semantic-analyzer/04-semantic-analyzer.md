# Семантический анализатор
Семантический анализатор проводит контекстный анализ (Context-sensitive analysis)
выполнения неформальных правил языка.

В процессе работы необходимо проверять правила:
1. Идентификаторы не могут быть описаны более одного раза в рамках
одной области видимости (но в разных - возможно, это называется
shadowing или masking);
2. Для каждого используемого идентификатора должно существовать
определяющее вхождение в текущей области видимости или выше;
3. Соответствие типов данных;

Для реализации потребуется хранить информацию об идентификаторах
и иметь доступ к областям видимости.

В мультипроходных компиляторах часто семантический анализ может
работать как отдельный проход. Так как в синтаксическом анализаторе
было выбрано промежуточное представление в виде дерева, будем
осуществлять семантический анализ по ходу формирования вершин
дерева.

## Реализация
Для реализации анализатора были созданы отдельные структуры.
```rust
#[derive(PartialEq, Clone)]
pub enum Usage {
    Constant(String),
    Type(Option<String>),
    Program,
    Variable(String),
    // Procedure, function…
}

pub struct Scope {
    identifiers: HashMap<String, Usage>,
}
```

Идентификаторы будем хранить в хэш-таблице, где ключ - сам
идентификатор, а значение - это тип использования. Воспользуемся
возможностью параметризовать enum и добавим поля к значениям
перечислимого типа. Для констант и переменных это будет идентификатор
типа данных, для типов данных - опциональный "родительский" тип (
в работе реализовались только type alias'ы, в более сложно случае
потребовалась полноценная отдельная таблица типов).

Сами области видимости будем хранить в синтаксическом анализаторе
в виде стека. При спуске в область видимости ниже выполняется
метод `analyzer.enter_scope()`, на выходе - `leave_scope()`.

Также добавим атрибуты к узлам дерева (такой подход в "Engineering a Compiler"
называется "Attribute Grammar Framework), таким как термы, выражения
и т.п. Будем заполнять атрибуты в процессе построения, чтобы
при формировании родительской вершины сразу проверять соответствие
типов и назначать тип результата.

Была реализована структура Analyzer с вспомогательными методами:
```rust
pub struct Analyzer {
    scopes: Vec<Scope>,
}

impl Analyzer {
    pub fn new() -> Self {}

    pub fn enter_scope(&mut self) {}

    pub fn leave_scope(&mut self) {}

    pub fn check_var_declaration(
        &mut self,
        decl: VarDeclaration,
    ) -> Result<VarDeclaration, CompilerError> {}

    pub fn check_type_declaration(
        &mut self,
        decl: TypeDeclaration,
    ) -> Result<TypeDeclaration, CompilerError> {}

    pub fn get_factor_type(&self, f: &Factor) -> Result<Usage, CompilerError> {}

    pub fn find_identifier(&self, id: &Identifier) -> Result<&Usage, CompilerError> {}

    pub fn merge_types(
        &self,
        type1: &String,
        type2: &String,
        pos: (usize, usize),
        strong: bool,
    ) -> Result<String, CompilerError> {}

    pub fn get_sub_term_type(&self, sub_term: &SubTerm) -> Result<String, CompilerError> {}

    pub fn check_expr(
        &self,
        e: &Expression,
        type_name: &String,
        pos: (usize, usize),
    ) -> Result<(), CompilerError> {}

    pub fn check_assignment(&self, a: VarAssignment) -> Result<VarAssignment, CompilerError> {}

```

## Тестирование
```pascal
program Test;
var
    x : myType;
begin
    x := 25;
end.
```

```text
Semantic Error [3:15] Unknown identifier "myType"
Semantic Error [5:6] Unknown identifier "x"
```
Так как тип неизвестен, переменная тоже не была добавлена.
Получаем сразу 2 семантических ошибки.

```pascal
program Test;
var
    x : integer;
    y : char;
begin
    x := 25 + y;
end.
```
```text
Semantic Error [6:16] Type mismatch
```

```pascal
program Test;
var
    x, y : integer;
begin
    while 25 + y do
        y := x + y;
end.
```
```text
Semantic Error [5:20] Expected boolean type
```

```pascal
program Test;
var
    x, y : integer;
begin
    if 25 then
        x := 5;
    else
        x := 26;
end.
```

if инвалидировался, else остался один
```text
Semantic Error [5:15] Expected boolean type
Syntax Error [7:9] Illegal statement
```

```pascal
program Test;
var
    x, y : integer;
    y, z : real;
begin
    y := 25;
end.
```

```text
Semantic Error [4:6] Redeclaration of "y"
```

```pascal
program Test;
type
    myType : integer;
    myType : char;
var
    x, y : integer;
    y, z : myType;
begin
    y := 25;
end.
```

```text
Semantic Error [4:11] Redeclaration of "myType"
Semantic Error [7:6] Redeclaration of "y"
```

## Пример полностью разобранной программы
```pascal
program Test;
type
    myType : integer;
var
    x, y : integer;
    b : boolean;
    z : myType;
begin
    x := 0;

    while x < 25 do
        x := x + 1;

    if x = 25 then
        y := 42 - 25 * (3 + 25 * (3 - 1));
    else if x = 26 then
        b := true or false and true;
    else
        y := 3;
end.
```
```text
Parsed program!
Errors:
Semantic Error [16:10] Type mismatch
Program {
    identifier: Identifier {
        name: Token {
            type: Identifier(
                "Test",
            ),
            position: (
                1,
                13,
            ),
        },
    },
    var_section: Some(
        VarSection {
            declarations: [
                VarDeclaration {
                    id: Identifier {
                        name: Token {
                            type: Identifier(
                                "x",
                            ),
                            position: (
                                5,
                                6,
                            ),
                        },
                    },
                    type_name: Identifier {
                        name: Token {
                            type: Identifier(
                                "integer",
                            ),
                            position: (
                                5,
                                19,
                            ),
                        },
                    },
                },
                VarDeclaration {
                    id: Identifier {
                        name: Token {
                            type: Identifier(
                                "y",
                            ),
                            position: (
                                5,
                                9,
                            ),
                        },
                    },
                    type_name: Identifier {
                        name: Token {
                            type: Identifier(
                                "integer",
                            ),
                            position: (
                                5,
                                19,
                            ),
                        },
                    },
                },
                VarDeclaration {
                    id: Identifier {
                        name: Token {
                            type: Identifier(
                                "z",
                            ),
                            position: (
                                6,
                                6,
                            ),
                        },
                    },
                    type_name: Identifier {
                        name: Token {
                            type: Identifier(
                                "myType",
                            ),
                            position: (
                                6,
                                15,
                            ),
                        },
                    },
                },
            ],
        },
    ),
    type_section: Some(
        TypeSection {
            declarations: [
                TypeDeclaration {
                    id: Identifier {
                        name: Token {
                            type: Identifier(
                                "myType",
                            ),
                            position: (
                                3,
                                11,
                            ),
                        },
                    },
                    definition: Identifier {
                        name: Token {
                            type: Identifier(
                                "integer",
                            ),
                            position: (
                                3,
                                21,
                            ),
                        },
                    },
                },
            ],
        },
    ),
    compound: Compound {
        statements: [
            Simple Statement {
                value: VarAssignment {
                    identifier: Identifier {
                        name: Token {
                            type: Identifier(
                                "x",
                            ),
                            position: (
                                8,
                                6,
                            ),
                        },
                    },
                    value: Simple SimpleExpression {
                        term: Term {
                            factor: Factor<Int>(Token { type: Integer(0), position: (8, 11) }),
                            sub_term: None,
                            term_type: "integer",
                        },
                        sub_expr: None,
                        expr_type: "integer",
                    },
                },
            },
            WhileLoop {
                value: WhileLoop {
                    condition: Rel RelationalExpression {
                        first: SimpleExpression {
                            term: Term {
                                factor: Factor<Variable>(Identifier { name: Token { type: Identifier("x"), position: (10, 12) } }),
                                sub_term: None,
                                term_type: "integer",
                            },
                            sub_expr: None,
                            expr_type: "integer",
                        },
                        op: <,
                        second: SimpleExpression {
                            term: Term {
                                factor: Factor<Int>(Token { type: Integer(25), position: (10, 17) }),
                                sub_term: None,
                                term_type: "integer",
                            },
                            sub_expr: None,
                            expr_type: "integer",
                        },
                    },
                    statement: Simple Statement {
                        value: VarAssignment {
                            identifier: Identifier {
                                name: Token {
                                    type: Identifier(
                                        "x",
                                    ),
                                    position: (
                                        11,
                                        10,
                                    ),
                                },
                            },
                            value: Simple SimpleExpression {
                                term: Term {
                                    factor: Factor<Variable>(Identifier { name: Token { type: Identifier("x"), position: (11, 15) } }),
                                    sub_term: None,
                                    term_type: "integer",
                                },
                                sub_expr: Some(
                                    SubExpression {
                                        op: Plus <+>,
                                        term: Term {
                                            factor: Factor<Int>(Token { type: Integer(1), position: (11, 19) }),
                                            sub_term: None,
                                            term_type: "integer",
                                        },
                                        sub_expr: None,
                                        sub_expr_type: "integer",
                                    },
                                ),
                                expr_type: "integer",
                            },
                        },
                    },
                },
            },
            Simple Statement {
                value: VarAssignment {
                    identifier: Identifier {
                        name: Token {
                            type: Identifier(
                                "y",
                            ),
                            position: (
                                18,
                                10,
                            ),
                        },
                    },
                    value: Simple SimpleExpression {
                        term: Term {
                            factor: Factor<Int>(Token { type: Integer(3), position: (18, 15) }),
                            sub_term: None,
                            term_type: "integer",
                        },
                        sub_expr: None,
                        expr_type: "integer",
                    },
                },
            },
        ],
    },
}

Process finished with exit code 0
```