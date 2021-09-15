use crate::token;
use crate::token::Token;

#[derive(Debug, PartialEq)]
pub enum Error {
    NoCloseDoubleQuote,
    Eos,
    TokenErr(token::Error),
}

#[derive(Debug, PartialEq)]
pub struct Variable {
    name: String,
    value: String,
}

/// Reinterpreted token for parser.
#[derive(Debug, PartialEq)]
pub enum Word {
    And,
    Or,
    Pipe,
    Terminator,
    String(String),
    Variable(Variable),
}

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    tokenizer: token::Tokenizer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(s: &'a str) -> Parser<'a> {
        Parser {
            tokenizer: token::Tokenizer::new(s),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Word>, Error> {
        let mut words = Vec::new();

        loop {
            match self.next_word() {
                Ok(word) => words.push(word),
                Err(Error::Eos) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(words)
    }

    fn next_word(&mut self) -> Result<Word, Error> {
        let mut value = String::new();
        let mut is_somethihg_found = false;

        loop {
            match self.peek_token() {
                Ok(Token::Ampersand) => {
                    let _ = self.next();

                    if self.eat_token(Token::Ampersand) {
                        let _ = self.next();
                        return Ok(Word::And);
                    } else {
                        return Ok(Word::Terminator);
                    }
                }
                Ok(Token::VerticalBar) => {
                    let _ = self.next();

                    if self.eat_token(Token::VerticalBar) {
                        let _ = self.next();
                        return Ok(Word::Or);
                    } else {
                        return Ok(Word::Pipe);
                    }
                }
                Ok(Token::Newline | Token::Semicolon) => {
                    if is_somethihg_found {
                        break;
                    } else {
                        let _ = self.next();
                        return Ok(Word::Terminator);
                    }
                }
                Ok(Token::Spaces { .. }) => {
                    let _ = self.next();

                    if is_somethihg_found {
                        break;
                    } else {
                        continue;
                    }
                }
                Ok(Token::DoubleQuote) => {
                    let _ = self.next();

                    match self.double_quoted_string() {
                        Ok(s) => return Ok(Word::String(s)),
                        Err(e) => return Err(e),
                    }
                }
                Ok(token @ Token::String { .. }) => {
                    let _ = self.next();

                    let name = String::from(token);

                    if is_variable_name(&name) && self.eat_token(Token::Equal) {
                        let value = self.value()?;

                        return Ok(Word::Variable(Variable { name, value }));
                    } else {
                        value.push_str(name.as_str());
                    }
                }
                Ok(token) => {
                    let _ = self.next();

                    value.push_str(String::from(token).as_str());
                }
                Err(token::Error::Eos) => {
                    let _ = self.next();

                    if is_somethihg_found {
                        break;
                    } else {
                        return Err(Error::Eos);
                    }
                }
                Err(e) => return Err(Error::TokenErr(e)),
            }

            is_somethihg_found = true;
        }

        Ok(Word::String(value))
    }

    fn value(&mut self) -> Result<String, Error> {
        let mut s = String::new();

        loop {
            match self.peek_token() {
                Ok(Token::DoubleQuote) => {
                    let _ = self.next();
                    let s = self.double_quoted_string()?;
                    return Ok(s);
                }
                Ok(
                    Token::Newline
                    | Token::Semicolon
                    | Token::Spaces { .. }
                    | Token::Ampersand
                    | Token::VerticalBar,
                ) => break,
                Ok(token) => {
                    let _ = self.next();
                    s.push_str(String::from(token).as_str());
                }
                Err(e) => return Err(Error::TokenErr(e)),
            }
        }

        Ok(s)
    }

    fn double_quoted_string(&mut self) -> Result<String, Error> {
        let mut s = String::new();

        loop {
            match self.next() {
                Ok(Token::DoubleQuote) => return Ok(s),
                Ok(token) => {
                    s.push_str(String::from(token).as_str());
                }
                Err(token::Error::Eos) => return Err(Error::NoCloseDoubleQuote),
                Err(e) => return Err(Error::TokenErr(e)),
            }
        }
    }

    fn next(&mut self) -> Result<token::Token<'a>, token::Error> {
        self.tokenizer.next()
    }

    fn peek_token(&mut self) -> Result<token::Token<'a>, token::Error> {
        self.tokenizer.peek_token()
    }

    fn eat_token(&mut self, token: token::Token) -> bool {
        self.tokenizer.eat_token(token)
    }
}

fn is_variable_name(s: &str) -> bool {
    for (i, c) in s.char_indices() {
        if i == 0 && c.is_ascii_digit() {
            return false;
        }

        if !(c.is_alphanumeric() || c == '_') {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_variable_name() {
        assert!(is_variable_name(&String::from("LS_COLORS")));
        assert!(is_variable_name(&String::from("var")));
        assert!(!is_variable_name(&String::from("1var")));
        assert!(!is_variable_name(&String::from("var-var")));
    }

    #[test]
    fn test_parser() {
        let s = "";
        let mut parser = Parser::new(s);

        assert_eq!(parser.parse(), Ok(Vec::new()));

        let s = "   ls   ";
        let mut parser = Parser::new(s);

        assert_eq!(parser.parse(), Ok(vec![Word::String(String::from("ls"))]));

        let s = "  LS_COLOR='*.rs=38;5;81'    ls -ABFHhov    --color=auto  ";

        let mut parser = Parser::new(s);

        assert_eq!(
            parser.parse(),
            Ok(vec![
                Word::Variable(Variable {
                    name: String::from("LS_COLOR"),
                    value: String::from("*.rs=38;5;81")
                }),
                Word::String(String::from("ls")),
                Word::String(String::from("-ABFHhov")),
                Word::String(String::from("--color=auto"))
            ])
        );

        let s = "ls | grep -i file && head || tail";

        let mut parser = Parser::new(s);

        assert_eq!(
            parser.parse(),
            Ok(vec![
                Word::String(String::from("ls")),
                Word::Pipe,
                Word::String(String::from("grep")),
                Word::String(String::from("-i")),
                Word::String(String::from("file")),
                Word::And,
                Word::String(String::from("head")),
                Word::Or,
                Word::String(String::from("tail")),
            ])
        );
    }
}
