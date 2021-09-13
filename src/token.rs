use std::ops::Range;
use std::str;

#[derive(Debug, PartialEq)]
pub enum Error {
    MissingEscapedChar,
    EOS,
}

#[derive(Debug, PartialEq)]
pub enum StringKind {
    SingleQuote,
    DoubleQuote,
    Raw,
}

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    // TODO: ;
    Equal,
    Newline,
    Spaces,
    String {
        s: &'a str,
        range: Range<usize>,
        kind: StringKind,
    },
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
        if self.eat_spaces() {
            return Ok(Token::Spaces);
        }

        let start = self.current();

        match self.one() {
            Some((_, '=')) => Ok(Token::Equal),
            Some((_, '\n')) => Ok(Token::Newline),
            Some((_, '\'')) => {
                let end = self.single_quoted_string()?;
                Ok(Token::String {
                    s: self.input,
                    range: (start + 1)..end,
                    kind: StringKind::SingleQuote,
                })
            }
            Some((_, '"')) => {
                let end = self.double_quoted_string()?;
                Ok(Token::String {
                    s: self.input,
                    range: (start + 1)..end,
                    kind: StringKind::DoubleQuote,
                })
            }
            Some(_) => {
                let end = self.raw_string()?;
                Ok(Token::String {
                    s: self.input,
                    range: start..end,
                    kind: StringKind::Raw,
                })
            }
            None => Err(Error::EOS),
        }
    }

    fn double_quoted_string(&mut self) -> Result<usize, Error> {
        loop {
            match self.one() {
                Some((i, '"')) => return Ok(i),
                Some((_, '\\')) => {
                    // Skip an escaped char.
                    match self.one() {
                        Some(_) => {}
                        None => return Err(Error::MissingEscapedChar),
                    }
                }
                Some(_) => {}
                None => return Err(Error::EOS),
            }
        }
    }

    fn single_quoted_string(&mut self) -> Result<usize, Error> {
        loop {
            match self.one() {
                Some((i, '\'')) => return Ok(i),
                Some(_) => continue,
                None => return Err(Error::EOS),
            }
        }
    }

    fn raw_string(&mut self) -> Result<usize, Error> {
        loop {
            match self.peek_one() {
                Some((i, c)) if "'\" \t\n=".contains(c) => return Ok(i),
                Some((_, '\\')) => {
                    // Skip a \.
                    self.one();
                    // Skip an escaped char.
                    if let None = self.one() {
                        return Err(Error::MissingEscapedChar);
                    }
                }
                Some(_) => {
                    self.one();
                }
                None => return Ok(self.current()),
            }
        }
    }

    fn eat_spaces(&mut self) -> bool {
        let mut eat = || self.eatc(' ') || self.eatc('\t');

        if !eat() {
            return false;
        }

        while eat() {}
        true
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

        assert_eq!(tokenizer.next(), Err(Error::EOS),);

        let s = "echo   ";
        let mut tokenizer = Tokenizer::new(s);

        assert_eq!(
            tokenizer.next(),
            Ok(Token::String {
                s,
                range: 0..4,
                kind: StringKind::Raw
            })
        );
        assert_eq!(tokenizer.next(), Ok(Token::Spaces));
        assert_eq!(tokenizer.next(), Err(Error::EOS));

        let s = r#"  LS_COLORS='*.rs=38;5;81' var=hoge   haskellorls   '-ABFHhov'   "--color=auto"  --time-style=iso"#;
        let mut tokenizer = Tokenizer::new(s);

        assert_eq!(tokenizer.next(), Ok(Token::Spaces));
        // LS_COLORS
        assert_eq!(
            tokenizer.next(),
            Ok(Token::String {
                s,
                range: 2..11,
                kind: StringKind::Raw
            })
        );
        assert_eq!(tokenizer.next(), Ok(Token::Equal));
        // '*.rs=38;5;81'
        assert_eq!(
            tokenizer.next(),
            Ok(Token::String {
                s,
                range: 13..25,
                kind: StringKind::SingleQuote
            })
        );
        assert_eq!(tokenizer.next(), Ok(Token::Spaces));
        // var
        assert_eq!(
            tokenizer.next(),
            Ok(Token::String {
                s,
                range: 27..30,
                kind: StringKind::Raw
            })
        );
        assert_eq!(tokenizer.next(), Ok(Token::Equal));
        // hoge
        assert_eq!(
            tokenizer.next(),
            Ok(Token::String {
                s,
                range: 31..35,
                kind: StringKind::Raw
            })
        );
        assert_eq!(tokenizer.next(), Ok(Token::Spaces));
        // haskellorls
        assert_eq!(
            tokenizer.next(),
            Ok(Token::String {
                s,
                range: 38..49,
                kind: StringKind::Raw
            })
        );
        assert_eq!(tokenizer.next(), Ok(Token::Spaces));
        // '-ABFHhov'
        assert_eq!(
            tokenizer.next(),
            Ok(Token::String {
                s,
                range: 53..61,
                kind: StringKind::SingleQuote
            })
        );
        assert_eq!(tokenizer.next(), Ok(Token::Spaces));
        // "--color=auto"
        assert_eq!(
            tokenizer.next(),
            Ok(Token::String {
                s,
                range: 66..78,
                kind: StringKind::DoubleQuote
            })
        );
        assert_eq!(tokenizer.next(), Ok(Token::Spaces));
        // --time-style
        assert_eq!(
            tokenizer.next(),
            Ok(Token::String {
                s,
                range: 81..93,
                kind: StringKind::Raw
            })
        );
        assert_eq!(tokenizer.next(), Ok(Token::Equal));
        // iso
        assert_eq!(
            tokenizer.next(),
            Ok(Token::String {
                s,
                range: 94..97,
                kind: StringKind::Raw
            })
        );
        assert_eq!(tokenizer.next(), Err(Error::EOS),);
    }
}
