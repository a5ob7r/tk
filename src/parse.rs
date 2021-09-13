use crate::token;

#[derive(Debug, PartialEq)]
pub enum Error {
    OrphanEqual,
    TokenErr(token::Error),
}

#[derive(Debug, PartialEq)]
pub enum Syntax {
    Variable { name: String, value: String },
    Command(String),
    Argument(String),
}

#[derive(Debug)]
pub struct Parser<'a> {
    tokenizer: token::Tokenizer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(s: &'a str) -> Parser<'a> {
        Parser {
            tokenizer: token::Tokenizer::new(s),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Syntax>, Error> {
        let mut is_found_command = false;
        let mut vec = Vec::new();

        loop {
            match self.next() {
                Ok(token::Token::String { s, range, kind }) => {
                    let mut name = String::from(&s[range]);

                    if kind == token::StringKind::Raw
                        && is_variable_name(&name)
                        && self.eat_token(token::Token::Equal)
                    {
                        let value = self.value()?;
                        vec.push(Syntax::Variable { name, value });
                        continue;
                    }
                    let value = self.value()?;
                    name.push_str(value.as_str());

                    if is_found_command {
                        vec.push(Syntax::Argument(name));
                    } else {
                        vec.push(Syntax::Command(name));
                        is_found_command = true;
                    }
                }
                Ok(token::Token::Equal) => return Err(Error::OrphanEqual),
                Err(token::Error::EOS) => break,
                _ => continue,
            }
        }

        Ok(vec)
    }

    fn value(&mut self) -> Result<String, Error> {
        let mut value = String::new();

        loop {
            match self.next() {
                Ok(token::Token::Equal) => value.push('='),
                Ok(token::Token::Newline) => break,
                Ok(token::Token::Spaces) => break,
                Ok(token::Token::String { s, range, .. }) => value.push_str(&s[range]),
                Err(err) => return Err(Error::TokenErr(err)),
            }
        }

        Ok(value)
    }

    fn next(&mut self) -> Result<token::Token<'a>, token::Error> {
        self.tokenizer.next()
    }

    fn eat_token(&mut self, token: token::Token) -> bool {
        self.tokenizer.eat_token(token)
    }
}

fn is_variable_name(s: &String) -> bool {
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

        assert_eq!(
            parser.parse(),
            Ok(vec![Syntax::Command(String::from("ls"))])
        );

        let s = "  LS_COLOR='*.rs=38;5;81'    ls -ABFHhov    --color=auto  ";

        let mut parser = Parser::new(s);

        assert_eq!(
            parser.parse(),
            Ok(vec![
                Syntax::Variable {
                    name: String::from("LS_COLOR"),
                    value: String::from("*.rs=38;5;81")
                },
                Syntax::Command(String::from("ls")),
                Syntax::Argument(String::from("-ABFHhov")),
                Syntax::Argument(String::from("--color=auto")),
            ])
        );
    }
}
