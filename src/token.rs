use std::convert::From;
use std::convert::TryFrom;
use std::ops::Range;
use std::str;

#[derive(Debug, PartialEq)]
pub enum Error {
    MissingEscapedChar,
    Eos,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    Ampersand,
    Asterisk,
    CloseBrace,
    CloseBracket,
    CloseParenthesis,
    DoubleQuote,
    Dollar,
    Equal,
    GreaterThan,
    LesserThan,
    Newline,
    OpenBrace,
    OpenBracket,
    OpenParenthesis,
    Semicolon,
    Tilda,
    VerticalBar,

    Spaces { s: &'a str, range: Range<usize> },
    String { s: &'a str, range: Range<usize> },
    QuotedString { s: &'a str, range: Range<usize> },
}

impl TryFrom<Token<'_>> for char {
    type Error = &'static str;

    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token {
            Token::Ampersand => Ok('&'),
            Token::Asterisk => Ok('*'),
            Token::CloseBrace => Ok('}'),
            Token::CloseBracket => Ok(']'),
            Token::CloseParenthesis => Ok(')'),
            Token::DoubleQuote => Ok('"'),
            Token::Dollar => Ok('$'),
            Token::Equal => Ok('='),
            Token::GreaterThan => Ok('>'),
            Token::LesserThan => Ok('<'),
            Token::Newline => Ok('\n'),
            Token::OpenBrace => Ok('{'),
            Token::OpenBracket => Ok('['),
            Token::OpenParenthesis => Ok('('),
            Token::Semicolon => Ok(';'),
            Token::Tilda => Ok('~'),
            Token::VerticalBar => Ok('|'),
            _ => Err("No mapped character."),
        }
    }
}

impl TryFrom<char> for Token<'_> {
    type Error = &'static str;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '&' => Ok(Token::Ampersand),
            '*' => Ok(Token::Asterisk),
            '}' => Ok(Token::CloseBrace),
            ']' => Ok(Token::CloseBracket),
            ')' => Ok(Token::CloseParenthesis),
            '"' => Ok(Token::DoubleQuote),
            '$' => Ok(Token::Dollar),
            '=' => Ok(Token::Equal),
            '>' => Ok(Token::GreaterThan),
            '<' => Ok(Token::LesserThan),
            '\n' => Ok(Token::Newline),
            '{' => Ok(Token::OpenBrace),
            '[' => Ok(Token::OpenBracket),
            '(' => Ok(Token::OpenParenthesis),
            ';' => Ok(Token::Semicolon),
            '~' => Ok(Token::Tilda),
            '|' => Ok(Token::VerticalBar),
            _ => Err("No mapped token."),
        }
    }
}

impl From<Token<'_>> for String {
    fn from(token: Token) -> String {
        match token {
            Token::Spaces { s, range }
            | Token::String { s, range }
            | Token::QuotedString { s, range } => s[range].to_string(),
            _ => String::from(char::try_from(token).unwrap()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tokenizer<'a> {
    input: &'a str,
    chars: str::CharIndices<'a>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(s: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            input: s,
            chars: s.char_indices(),
        }
    }

    pub fn eat_token(&mut self, token: Token) -> bool {
        match self.peek_token() {
            Ok(t) if t == token => {
                let _ = self.next();
                true
            }
            _ => false,
        }
    }

    pub fn peek_token(&mut self) -> Result<Token<'a>, Error> {
        self.clone().next()
    }

    pub fn next(&mut self) -> Result<Token<'a>, Error> {
        match self.one() {
            Some((start, '\'')) => {
                let end = self.single_quoted_string()?;
                Ok(Token::QuotedString {
                    s: self.input,
                    range: (start + 1)..end,
                })
            }
            Some((start, ' ' | '\t')) => {
                let end = self.spaces()?;
                Ok(Token::Spaces {
                    s: self.input,
                    range: start..end,
                })
            }
            Some((start, c)) => Token::try_from(c).or_else(|_| {
                let end = self.raw_string()?;
                Ok(Token::String {
                    s: self.input,
                    range: start..end,
                })
            }),
            None => Err(Error::Eos),
        }
    }

    fn single_quoted_string(&mut self) -> Result<usize, Error> {
        loop {
            match self.one() {
                Some((i, '\'')) => return Ok(i),
                Some(_) => continue,
                None => return Err(Error::Eos),
            }
        }
    }

    fn raw_string(&mut self) -> Result<usize, Error> {
        loop {
            match self.peek_one() {
                Some((_, '\\')) => {
                    // Skip a \.
                    self.one();
                    // Skip an escaped char.
                    if self.one().is_none() {
                        return Err(Error::MissingEscapedChar);
                    }
                }
                Some((i, c)) => match Token::try_from(c) {
                    Err(_) if !" \t".contains(c) => {
                        self.one();
                    }
                    _ => return Ok(i),
                },
                None => return Ok(self.current()),
            }
        }
    }

    fn spaces(&mut self) -> Result<usize, Error> {
        while self.eatc(' ') || self.eatc('\t') {}
        Ok(self.current())
    }

    fn eatc(&mut self, c: char) -> bool {
        match self.peek_one() {
            Some((_, ch)) if ch == c => {
                self.one();
                true
            }
            _ => false,
        }
    }

    fn current(&self) -> usize {
        self.peek_one().map(|(i, _)| i).unwrap_or(self.input.len())
    }

    fn one(&mut self) -> Option<(usize, char)> {
        self.chars.next()
    }

    fn peek_one(&self) -> Option<(usize, char)> {
        self.chars.clone().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer() {
        let s = "";
        let mut tokenizer = Tokenizer::new(s);

        assert_eq!(tokenizer.next(), Err(Error::Eos),);

        let s = "echo   ";
        let mut tokenizer = Tokenizer::new(s);

        assert_eq!(tokenizer.next(), Ok(Token::String { s, range: 0..4 }));
        assert_eq!(tokenizer.next(), Ok(Token::Spaces { s, range: 4..7 }));
        assert_eq!(tokenizer.next(), Err(Error::Eos));

        let s = r#"  LS_COLORS='*.rs=38;5;81' var=hoge   haskellorls   '-ABFHhov'   "--color=auto"  --time-style=iso"#;
        let mut tokenizer = Tokenizer::new(s);

        assert_eq!(tokenizer.next(), Ok(Token::Spaces { s, range: 0..2 }));
        // LS_COLORS
        assert_eq!(tokenizer.next(), Ok(Token::String { s, range: 2..11 }));
        assert_eq!(tokenizer.next(), Ok(Token::Equal));
        // '*.rs=38;5;81'
        assert_eq!(
            tokenizer.next(),
            Ok(Token::QuotedString { s, range: 13..25 })
        );
        assert_eq!(tokenizer.next(), Ok(Token::Spaces { s, range: 26..27 }));
        // var
        assert_eq!(tokenizer.next(), Ok(Token::String { s, range: 27..30 }));
        assert_eq!(tokenizer.next(), Ok(Token::Equal));
        // hoge
        assert_eq!(tokenizer.next(), Ok(Token::String { s, range: 31..35 }));
        assert_eq!(tokenizer.next(), Ok(Token::Spaces { s, range: 35..38 }));
        // haskellorls
        assert_eq!(tokenizer.next(), Ok(Token::String { s, range: 38..49 }));
        assert_eq!(tokenizer.next(), Ok(Token::Spaces { s, range: 49..52 }));
        // '-ABFHhov'
        assert_eq!(
            tokenizer.next(),
            Ok(Token::QuotedString { s, range: 53..61 })
        );
        assert_eq!(tokenizer.next(), Ok(Token::Spaces { s, range: 62..65 }));
        // "--color=auto"
        assert_eq!(tokenizer.next(), Ok(Token::DoubleQuote));
        assert_eq!(tokenizer.next(), Ok(Token::String { s, range: 66..73 }));
        assert_eq!(tokenizer.next(), Ok(Token::Equal));
        assert_eq!(tokenizer.next(), Ok(Token::String { s, range: 74..78 }));
        assert_eq!(tokenizer.next(), Ok(Token::DoubleQuote));
        assert_eq!(tokenizer.next(), Ok(Token::Spaces { s, range: 79..81 }));
        // --time-style
        assert_eq!(tokenizer.next(), Ok(Token::String { s, range: 81..93 }));
        assert_eq!(tokenizer.next(), Ok(Token::Equal));
        // iso
        assert_eq!(tokenizer.next(), Ok(Token::String { s, range: 94..97 }));
        assert_eq!(tokenizer.next(), Err(Error::Eos),);
    }
}
