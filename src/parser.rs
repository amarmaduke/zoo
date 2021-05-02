
use std::collections::VecDeque;

use crate::term::{Sort, Bind, Term};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Token {
    Star,
    Box,
    Var(usize, usize),
    Lambda,
    ForAll,
    Dot,
    Colon,
    OpenP,
    CloseP,
    None,
}

struct Parser<'a> {
    text : &'a str,
    tokens : VecDeque<Token>,
    context : Vec<(usize, usize)>
}

impl<'a> Parser<'a> {

    fn tokenize(&mut self) {
        #[derive(Copy, Clone, PartialEq, Eq)]
        enum Mode { Variable, Constant }

        let mut mode = Mode::Constant;
        let mut result = VecDeque::with_capacity(self.text.len());
        let (mut i, mut start, mut end) = (0, 0, 0);

        for x in self.text.chars() {
            if !x.is_whitespace() {
                let token = match x {
                    '*' => Token::Star,
                    '#' => Token::Box,
                    '\\' => Token::Lambda,
                    '@' => Token::ForAll,
                    '.' => Token::Dot,
                    ':' => Token::Colon,
                    '(' => Token::OpenP,
                    ')' => Token::CloseP,
                    _ => Token::None
                };
                mode = match mode {
                    Mode::Constant => {
                        if token == Token::None {
                            start = i;
                            end = i + 1;
                            Mode::Variable
                        } else {
                            result.push_back(token);
                            Mode::Constant
                        }
                    },
                    Mode::Variable => {
                        if token == Token::None {
                            end += 1;
                            Mode::Variable
                        } else {
                            result.push_back(Token::Var(start, end));
                            result.push_back(token);
                            Mode::Constant
                        }
                    }
                };
            } else {
                if mode == Mode::Variable {
                    result.push_back(Token::Var(start, end));
                    mode = Mode::Constant;
                }
            }
            i += 1;
        }

        if mode == Mode::Variable {
            result.push_back(Token::Var(start, end));
        }

        self.tokens = result;
    }

    fn next(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }

    fn require(&mut self, token:Token) -> Result<(), String> {
        if let Some(t) = self.next() {
            if token == t {
                Ok(())
            } else {
                Err(format!("Parse Error: Missing required token {:?}", token))?
            }
        } else {
            Err("Parse Error: Out of tokens".to_owned())
        }
    }

    fn parse_var(&mut self, start:usize, end:usize) -> Result<Term, String> {
        let mut index = (self.context.len() as i32) - 1;
        for (i, j) in self.context.iter().rev() {
            match (self.text.get(*i..*j), self.text.get(start..end)) {
                (Some(x), Some(y)) if x == y =>
                    return Ok(Term::Variable { index }),
                _ => ()
            };
            index -= 1;
        }
        Err("Parse Error: Missing variable in context".to_owned())
    }

    fn parse_binder(&mut self, bind:Bind) -> Result<Term, String> {
        let var_token = self.next();
        if let Some(Token::Var(start, end)) = var_token {
            self.require(Token::Colon)?;
            let annotation = self.parse_expr()?;
            self.require(Token::Dot)?;
            self.context.push((start, end));
            let expr = self.parse_expr()?;
            self.context.pop();
            Ok(Term::Binder {
                bind,
                type_annotation:Box::new(annotation),
                body:Box::new(expr)
            })
        } else {
            Err("Parse Error: Missing variable in binder".to_owned())
        }
    }

    fn parse_expr(&mut self) -> Result<Term, String> {
        let mut result = vec![];

        loop {
            match self.next() {
                Some(Token::Star) => {
                    return Ok(Term::Constant(Sort::Type));
                },
                Some(Token::Box) => {
                    return Ok(Term::Constant(Sort::Type));
                },
                Some(Token::Var(start, end)) => {
                    let term = self.parse_var(start, end)?;
                    result.push(term);
                },
                Some(Token::Lambda) => {
                    let term = self.parse_binder(Bind::Term)?;
                    result.push(term);
                },
                Some(Token::ForAll) => {
                    let term = self.parse_binder(Bind::Type)?;
                    result.push(term);
                },
                Some(Token::OpenP) => {
                    let expr = self.parse_expr()?;
                    self.require(Token::CloseP)?;
                    result.push(expr);
                },
                Some(t) => {
                    self.tokens.push_front(t);
                    break
                },
                None => break,
            };
        }

        let initial = result.pop()
            .ok_or("Parse Error: No term in application")?;
        Ok(result.drain(..).fold(initial, |acc, t| {
            Term::Application {
                function:Box::new(acc),
                argument:Box::new(t)
            }
        }))
    }
}

pub fn parse<'a>(input: &'a str) -> Result<Term, String> {
    let mut parser = Parser {
        text:input,
        tokens: VecDeque::new(),
        context: vec![]
    };
    parser.tokenize();
    parser.parse_expr()
}
