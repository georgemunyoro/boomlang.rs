use phf::phf_map;
use std::fmt;

pub const DOUBLE_QUOTE: &str = "'";
pub const SINGLE_QUOTE: &str = "\"";

pub static BASE_TOKEN_IDS: phf::Map<&'static str, &'static str> = phf_map! {
    "," => "SYMB_COMMA",
    "(" => "OPEN_PAREN",
    ")" => "CLOS_PAREN",
    "{" => "OPEN_PAREN",
    "}" => "CLOS_PAREN",
    ";" => "CLOS_EXPRN",
    "=" => "OPER_EQUAL",
    "+" => "OPER_ADDTN",
    "-" => "OPER_MINUS",
    "*" => "OPER_MULTI",
    "%" => "OPER_MODUL",
    "/" => "OPER_FSLSH",
    "\\" => "SYMB_BSLSH",
    "==" => "OPER_IS_EQUAL",
    "!=" => "OPER_IS_NEQUAL",
    ">=" => "OPER_IS_MORE_EQUAL",
    "<=" => "OPER_IS_LESS_EQUAL",
    ">" => "OPER_IS_MORE",
    "<" => "OPER_IS_LESS",
    "||" => "OPER_OR",
    "&&" => "OPER_AND"
};

pub struct Source {
    pub contents: String,
}

#[derive(Clone)]
pub struct Token {
    pub id: String,
    pub value: String,
}

pub struct Lexer {
    source: Source,
    in_string: bool,
    in_number: bool,
    buffer: String,
    in_string_type: String,
    tokens: Vec<Token>,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}>{}</{}>", self.id, self.value, self.id)
    }
}

impl Token {
    pub fn new<T: AsRef<str>>(id: T, value: T) -> Self {
        Self {
            id: id.as_ref().to_string(),
            value: value.as_ref().to_string(),
        }
    }
}

impl Source {
    pub fn new(contents: &str) -> Self {
        Self {
            contents: String::from(contents),
        }
    }

    fn strip_comments(&self) -> String {
        let mut source_lines: Vec<String> = Vec::new();
        for line in self.contents.split("\n") {
            if !String::from(line).trim().starts_with("#") {
                source_lines.push(String::from(line).trim().to_string());
            }
        }
        return source_lines.join("\n");
    }
}

impl Lexer {
    pub fn new(source: Source) -> Self {
        Self {
            source,
            in_string: false,
            in_number: false,
            tokens: Vec::new(),
            in_string_type: String::from(""),
            buffer: String::from(""),
        }
    }

    pub fn get_tokens(&self) -> &Vec<Token> {
        return &self.tokens;
    }

    pub fn lex(&mut self) {
        let source: String = self.source.strip_comments();

        for i in source.chars() {
            self.buffer += &i.to_string();

            if self.in_string {
                self.handle_string_char(i, false);
                continue;
            }

            if self.in_number && !self.handle_digit_char(i, false) {
                continue;
            }

            if i.is_digit(10) {
                self.in_number = true;
                self.buffer = i.to_string();
                continue;
            }

            if i.to_string() == DOUBLE_QUOTE || i.to_string() == SINGLE_QUOTE {
                self.buffer = String::from("");
                self.in_string = true;
                self.in_string_type = i.to_string();
                continue;
            }

            if i.is_whitespace() {
                if !self
                    .buffer
                    .chars()
                    .take(self.buffer.len() - 1)
                    .collect::<String>()
                    .trim()
                    .is_empty()
                {
                    self.tokens.push(Token::new(
                        String::from("ITEM"),
                        self.buffer.chars().take(self.buffer.len() - 1).collect(),
                    ));
                }
                self.buffer = String::from("");
                continue;
            }

            if String::from("|=&").contains(i) {
                if BASE_TOKEN_IDS
                    .keys()
                    .find(|k| String::from(**k) == i.to_string())
                    != None
                {
                    if let Some(id) = BASE_TOKEN_IDS.get(&i.to_string()) {
                        self.tokens
                            .insert(self.tokens.len(), Token::new(id.to_string(), i.to_string()));
                    }
                }
                self.buffer = String::from("");
                continue;
            }

            if BASE_TOKEN_IDS
                .keys()
                .find(|k| String::from(**k) == i.to_string())
                != None
            {
                if !self.buffer.trim().is_empty() {
                    let buff_without_last_char: String =
                        self.buffer.chars().take(self.buffer.len() - 1).collect();
                    if !buff_without_last_char.trim().is_empty() {
                        self.tokens.push(Token::new(
                            String::from("ITEM"),
                            String::from(buff_without_last_char),
                        ));
                    }
                }

                self.tokens.push(Token::new(
                    BASE_TOKEN_IDS.get(&i.to_string()).unwrap().to_string(),
                    i.to_string(),
                ));
                self.buffer = String::from("");
                continue;
            }
        }

        if self.in_number {
            self.handle_digit_char(' ', true);
        }

        if self.in_string {
            self.handle_string_char(' ', true);
        }
    }

    /*
     * Handle digit characters if currently dealing with a number.
     *
     * Adds the number currently stored in the buffer to the token list,
     * clears the buffer, and returns true if the number has actually
     * ended. In which case the lexer will continue on to find out what
     * else the current character is.
     *
     * Does nothing and returns false if the character was in fact a number.
     */
    fn handle_digit_char(&mut self, i: char, include_last_char: bool) -> bool {
        if !i.is_digit(10) && i.to_string() != "." {
            self.in_number = false;
            self.tokens.push(Token::new(
                String::from("NUMBER_LIT"),
                if include_last_char {
                    self.buffer.clone()
                } else {
                    self.buffer.chars().take(self.buffer.len() - 1).collect()
                },
            ));
            self.buffer = String::from("");
            return true;
        }
        return false;
    }

    /*
     * Handle string literal characters if currently dealing with a string.
     *
     * Adds the string currently stored in the buffer to the token list,
     * clears the buffer, and returns true if the character was of the same
     * type used to open and string has actually ended.
     *
     * Does nothing and returns false if the character was in fact
     * still part of a string.
     */
    fn handle_string_char(&mut self, i: char, include_last_char: bool) -> bool {
        if i.to_string() != self.in_string_type {
            return false;
        }

        self.in_string = false;
        let mut t_val: String = self.buffer.chars().take(self.buffer.len() - 1).collect();
        self.tokens.push(Token::new(
            "STRING_LIT",
            if include_last_char {
                &self.buffer
            } else {
                &mut t_val
            },
        ));
        self.buffer = String::from("");
        return true;
    }
}
