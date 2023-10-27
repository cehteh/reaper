use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    Print,
    Fn,
    If,
    Else,
    Identifier,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Number,
    Plus,
    Minus,
    Star,
    Slash,
    Comma,
    Semicolon,
    Less,
    Return,
    Equal,
    Bang,
    BangEqual,
    DoubleEqual,
    True,
    False,
    Null,
    While,
    String,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub value: String,
}

impl Token {
    fn new(kind: TokenKind, value: &str) -> Token {
        Token {
            kind,
            value: value.to_string(),
        }
    }
}

pub struct Tokenizer<'a> {
    src: &'a str,
    start: usize,
}

impl Iterator for Tokenizer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let re_keyword = r"?P<keyword>print|fn|if|else|return|while";
        let re_literal = r"?P<literal>true|false|null";
        let re_identifier = r"?P<identifier>[a-zA-Z_][a-zA-Z0-9_]*";
        let re_individual = r"?P<individual>[-+*/(){};,<=!]";
        let re_double = r"?P<double>==|!=";
        let re_number = r"?P<number>[-+]?\d+(\.\d+)?";
        let re_string = r#"?P<string>"([^"]*)""#;

        let r = Regex::new(
            format!(
                "({})|({})|({})|({})|({})|({})|({})",
                re_keyword, re_literal, re_identifier, re_double, re_individual, re_number, re_string,
            )
            .as_str(),
        )
        .unwrap();

        let token = match r.captures_at(self.src, self.start) {
            Some(captures) => {
                if let Some(m) = captures.name("keyword") {
                    self.start = m.end();
                    match m.as_str() {
                        "print" => Token::new(TokenKind::Print, "print"),
                        "fn" => Token::new(TokenKind::Fn, "fn"),
                        "if" => Token::new(TokenKind::If, "if"),
                        "else" => Token::new(TokenKind::Else, "else"),
                        "return" => Token::new(TokenKind::Return, "return"),
                        "while" => Token::new(TokenKind::While, "return"),
                        _ => unreachable!(),
                    }
                } else if let Some(m) = captures.name("literal") {
                    self.start = m.end();
                    match m.as_str() {
                        "true" => Token::new(TokenKind::True, "true"),
                        "false" => Token::new(TokenKind::False, "false"),
                        "null" => Token::new(TokenKind::Null, "null"),
                        _ => unreachable!(),
                    }
                } else if let Some(m) = captures.name("identifier") {
                    self.start = m.end();
                    Token::new(TokenKind::Identifier, m.as_str())
                } else if let Some(m) = captures.name("double") {
                    self.start = m.end();
                    match m.as_str() {
                        "==" => Token::new(TokenKind::DoubleEqual, "=="),
                        "!=" => Token::new(TokenKind::BangEqual, "=="),
                        _ => unreachable!(),
                    }
                } else if let Some(m) = captures.name("individual") {
                    self.start = m.end();
                    match m.as_str() {
                        "(" => Token::new(TokenKind::LeftParen, "("),
                        ")" => Token::new(TokenKind::RightParen, ")"),
                        "{" => Token::new(TokenKind::LeftBrace, "{"),
                        "}" => Token::new(TokenKind::RightBrace, "}"),
                        "+" => Token::new(TokenKind::Plus, "+"),
                        "-" => Token::new(TokenKind::Minus, "-"),
                        "*" => Token::new(TokenKind::Star, "*"),
                        "/" => Token::new(TokenKind::Slash, "/"),
                        ";" => Token::new(TokenKind::Semicolon, ";"),
                        "," => Token::new(TokenKind::Comma, ","),
                        "<" => Token::new(TokenKind::Less, ","),
                        "=" => Token::new(TokenKind::Equal, ","),
                        "!" => Token::new(TokenKind::Bang, ","),
                        _ => unreachable!(),
                    }
                } else if let Some(m) = captures.name("number") {
                    self.start = m.end();
                    Token::new(TokenKind::Number, m.as_str())
                } else if let Some(m) = captures.name("string") {
                    self.start = m.end();
                    println!("TOKENIZED: {:?}", m.as_str());
                    Token::new(TokenKind::String, m.as_str())
                } else {
                    return None;
                }
            }
            None => return None,
        };

        Some(token)
    }
}

impl<'a> Tokenizer<'a> {
    pub fn new(src: &'a str) -> Tokenizer<'a> {
        Tokenizer { src, start: 0 }
    }
}
