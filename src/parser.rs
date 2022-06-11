#[path = "./lexer.rs"]
mod lexer;

use std::fmt;

#[derive(Clone)]
pub struct Data {
    pub value: String,
    pub data_type: String,
    pub int_val: i32,
    pub list_data: Vec<Data>,
}

impl Data {
    pub fn new(data_type: &str, value: &str) -> Self {
        Self {
            value: String::from(value),
            data_type: String::from(data_type),
            int_val: 0,
            list_data: Vec::new(),
        }
    }
}

pub trait Node {
    fn get_value(&self) -> Data;
    fn get_type(&self) -> &str;
    fn repr(&self) -> String;
}

pub struct UnaryNode {
    pub value: Data,
    pub name: String,
}

pub struct BinaryNode {
    pub value: Data,
    pub left: Box<dyn Node>,
    pub right: Box<dyn Node>,
}

pub struct ListNode {
    pub value: Vec<Box<dyn Node>>,
    pub name: String,
    pub index: usize,
}

impl Node for ListNode {
    fn get_value(&self) -> Data {
        if let Some(data_at_index) = self.value.get(self.index) {
            return data_at_index.get_value();
        }
        return Data::new("", "");
    }

    fn get_type(&self) -> &str {
        "list"
    }

    fn repr(&self) -> String {
        return String::from("List[]");
    }
}

impl Node for UnaryNode {
    fn get_value(&self) -> Data {
        self.value.clone()
    }

    fn get_type(&self) -> &str {
        "unary"
    }

    fn repr(&self) -> String {
        format!("{} -> {}", self.name, self.value)
    }
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Display for dyn Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[ {} ]", self.repr())
    }
}

impl Node for BinaryNode {
    fn get_value(&self) -> Data {
        self.value.clone()
    }

    fn get_type(&self) -> &str {
        "binary"
    }

    fn repr(&self) -> String {
        format!("{} <- {} -> {}", self.left, self.get_value(), self.right)
    }
}

impl UnaryNode {
    pub fn new(name: &str, value: Data) -> Self {
        Self {
            value,
            name: String::from(name),
        }
    }
}

impl BinaryNode {
    pub fn new(value: Data, left: Box<dyn Node>, right: Box<dyn Node>) -> Self {
        Self { left, right, value }
    }
}

pub struct Parser {
    lx: lexer::Lexer,
    pos: usize,
    running: bool,
}

impl Parser {
    pub fn new(source_contents: &str) -> Self {
        Self {
            lx: lexer::Lexer::new(lexer::Source::new(source_contents)),
            pos: 0,
            running: false,
        }
    }

    pub fn parse(&mut self) -> Vec<Box<dyn Node>> {
        let mut nodes: Vec<Box<dyn Node>> = Vec::new();

        for t in self.lx.get_tokens() {
            println!("{}", t);
        }

        self.running = true;
        while self.running {
            nodes.push(self.parse_expr());
        }
        return nodes;
    }

    fn parse_func_args(&mut self) -> Box<dyn Node> {
        let mut param_list_node = Box::new(ListNode {
            value: Vec::new(),
            name: String::from("PARAMS_LIST"),
            index: 0,
        });

        while self.get_curr_token().id != "CLOS_PAREN" {
            param_list_node.value.push(self.parse_expr());
            if self.get_curr_token().id == "SYMB_COMMA" {
                self.pos += 1;
                continue;
            } else {
                break;
            }
        }
        return param_list_node;
    }

    fn parse_expr(&mut self) -> Box<dyn Node> {
        self.running = false;

        println!("{}", self.get_curr_token());

        if self.get_curr_token().id == "ITEM" {
            if self.get_next_token().id == "OPEN_PAREN" {
                let mut n = BinaryNode::new(
                    Data::new("FUNC", "FUNC"),
                    Box::new(UnaryNode::new(
                        "ITEM",
                        Data::new("", &self.get_curr_token().value.to_string()),
                    )),
                    Box::new(UnaryNode::new("", Data::new("", ""))),
                );

                self.pos += 2;
                n.right = self.parse_func_args();

                if self.get_curr_token().id == "CLOS_PAREN" {
                    self.pos += 1;
                    return Box::new(n);
                } else {
                    if self.get_curr_token().id == "END" {
                        return Box::new(n);
                    } else {
                        panic!(
                            "Expected ')', found {} instead.",
                            self.get_curr_token().value,
                        );
                    }
                }
            } else if self.get_next_token().id.starts_with("OPER_") {
                let mut n = BinaryNode::new(
                    Data::new("ITEM", "ITEM"),
                    Box::new(UnaryNode::new(
                        "ITEM",
                        Data::new("", &self.get_curr_token().value.to_string()),
                    )),
                    Box::new(UnaryNode::new("", Data::new("", ""))),
                );
                self.pos += 2;
                n.right = self.parse_expr();
                return Box::new(n);
            } else {
                self.pos += 1;
                return Box::new(UnaryNode::new(
                    "ITEM",
                    Data::new("", &self.get_prev_token().value.to_string()),
                ));
            }
        } else if self.get_curr_token().id == "OPEN_PAREN" {
            self.pos += 1;
            let expr = self.parse_expr();
            self.pos += 1;
            return expr;
        } else if self.get_curr_token().id == "NUMBER_LIT"
            || self.get_curr_token().id == "STRING_LIT"
        {
            if self.get_next_token().id.starts_with("OPER_") {
                let mut n = BinaryNode::new(
                    Data::new(
                        &self.get_next_token().id.to_string(),
                        &self.get_next_token().id.to_string(),
                    ),
                    Box::new(UnaryNode::new(
                        &self.get_curr_token().id.to_string(),
                        Data::new("", &self.get_curr_token().value.to_string()),
                    )),
                    Box::new(UnaryNode::new("", Data::new("", ""))),
                );
                self.pos += 2;
                n.right = self.parse_expr();
                return Box::new(n);
            } else {
                self.pos += 1;
                return Box::new(UnaryNode::new(
                    &self.get_prev_token().id.to_string(),
                    Data::new("", &self.get_prev_token().value.to_string()),
                ));
            }
        }

        self.running = false;

        return Box::new(UnaryNode::new("END", Data::new("END", "END")));
    }

    fn get_prev_token(&self) -> lexer::Token {
        if self.pos > 0 {
            if let Some(prev_token) = self.lx.get_tokens().get(self.pos - 1) {
                return prev_token.clone();
            }
        }
        return lexer::Token::new("END", "END");
    }

    fn get_curr_token(&self) -> lexer::Token {
        if self.pos < self.lx.get_tokens().len() {
            if let Some(curr_token) = self.lx.get_tokens().get(self.pos) {
                return curr_token.clone();
            }
        }
        return lexer::Token::new("END", "END");
    }

    fn get_next_token(&self) -> lexer::Token {
        if self.pos < self.lx.get_tokens().len() - 1 {
            if let Some(next_token) = self.lx.get_tokens().get(self.pos + 1) {
                return next_token.clone();
            }
        }
        return lexer::Token::new("END", "END");
    }
}
