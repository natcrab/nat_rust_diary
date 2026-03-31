use std::iter::Peekable;
use std::str::{Chars, from_utf8_unchecked};
use std::vec::IntoIter;
use std::{fmt, io};
#[derive(PartialEq, Debug, Clone, Copy)]
enum Token {
    Num(f64),
    RightPar,
    LeftPar,
    Plus,
    Minus,
    Mult,
    Div,
    Pow,
    Equal,
    Greater,
    Less,
    BitFlip,
    EOF,
}

enum Expr {
    Num(f64),
    Unary {
        op: Token,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
}

impl Expr {
    fn interpret(self) -> f64 {
        match self {
            Expr::Num(n) => n,
            Expr::Unary { op, expr: e } => match op {
                Token::Minus => -(e.interpret()),
                Token::BitFlip => {
                    let mut n = e.interpret() as i64;
                    n = !n;
                    n as f64
                }
                _ => 0.0,
            },
            Expr::Binary {
                left: l,
                op,
                right: r,
            } => match op {
                Token::Minus => l.interpret() - r.interpret(),
                Token::Plus => l.interpret() + r.interpret(),
                Token::Mult => l.interpret() * r.interpret(),
                Token::Div => l.interpret() / r.interpret(),
                Token::Pow => l.interpret().powf(r.interpret()),
                Token::Equal => ((l.interpret() - r.interpret()).abs() < 1e-10) as i64 as f64,
                Token::Greater => (l.interpret() > r.interpret()) as i64 as f64,
                Token::Less => (l.interpret() < r.interpret()) as i64 as f64,
                _ => 0.0,
            },
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::BitFlip => write!(f, ""),
            Token::Num(n) => write!(f, "{}", n),
            Token::LeftPar => write!(f, "("),
            Token::RightPar => write!(f, ")"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Mult => write!(f, "*"),
            Token::Div => write!(f, "/"),
            Token::Pow => write!(f, "^"),
            Token::Less => write!(f, "<"),
            Token::Equal => write!(f, "="),
            Token::Greater => write!(f, ">"),
            Token::EOF => write!(f, ""),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Num(n) => write!(f, "{}", n),
            Expr::Unary { op, expr } => write!(f, "({} {})", op, expr),
            Expr::Binary { left, op, right } => write!(f, "({} {} {})", op, left, right),
        }
    }
}

struct Scanner<'a> {
    iter: Peekable<Chars<'a>>,
    num_buffer: Vec<u8>,
    tokens: Vec<Token>,
}

impl<'a> Scanner<'a> {
    fn new(source: &'a String) -> Result<Scanner<'a>, String> {
        let mut scanner = Scanner {
            iter: source.chars().peekable(),
            num_buffer: vec![],
            tokens: vec![],
        };
        scanner.scan_all()?;
        Ok(scanner)
    }

    fn scan_all(&mut self) -> Result<(), String> {
        match self.scan() {
            None => Err(format!("Illegal token").to_string()),
            Some(Token::EOF) => Ok(()),
            Some(t) => {
                self.add_token(t);
                self.scan_all()
            }
        }
    }

    fn advance(&mut self) -> Option<char> {
        self.iter.next()
    }

    fn scan(&mut self) -> Option<Token> {
        match self.advance() {
            None => Some(Token::EOF),
            Some('(') => Some(Token::LeftPar),
            Some(')') => Some(Token::RightPar),
            Some('-') => Some(Token::Minus),
            Some('+') => Some(Token::Plus),
            Some('>') => Some(Token::Greater),
            Some('~') => Some(Token::BitFlip),
            Some('<') => Some(Token::Less),
            Some('=') => Some(Token::Equal),
            Some('*') | Some('x') => Some(Token::Mult),
            Some('/') => Some(Token::Div),
            Some('^') => Some(Token::Pow),
            Some(c) => {
                if char::is_ascii_digit(&c) || c == '.' {
                    self.num_buffer.push(c as u8);
                    if self
                        .iter
                        .peek()
                        .is_some_and(|x| char::is_ascii_digit(x) || *x == '.')
                    {
                        self.scan()
                    } else {
                        if *self.num_buffer.last().unwrap() == b'.' {
                            return None;
                        }
                        unsafe {
                            let ret = Some(Token::Num(
                                from_utf8_unchecked(&self.num_buffer)
                                    .parse::<f64>()
                                    .unwrap(),
                            ));
                            self.num_buffer.clear();
                            ret
                        }
                    }
                } else if char::is_whitespace(c) {
                    self.scan()
                } else {
                    None
                }
            }
        }
    }

    fn add_token(&mut self, t: Token) {
        self.tokens.push(t);
    }
}

/*
* Grammar:
* expr -> comparison
* comparison -> term (">", "<") term
* term -> factor ("+"| "-") factor
* factor -> pow ("*" | "/") pow
* pow -> unary ("^") unary
* unary -> ("~" | "-") terminal
* terminal -> Num(n)
*/
struct Parser {
    iter: Peekable<IntoIter<Token>>,
}

impl Parser {
    fn new(tokens: Scanner) -> Parser {
        Parser {
            iter: tokens.tokens.into_iter().peekable(),
        }
    }

    /*fn is_end(&mut self) -> bool {
        match self.iter.peek() {
            None | Some(Token::EOF) => true,
            _ => false,
        }
    }*/

    fn parse_expression(&mut self) -> Result<Box<Expr>, String> {
        Ok(self.parse_comparison()?)
    }

    fn parse_comparison(&mut self) -> Result<Box<Expr>, String> {
        let lhs = self.parse_term()?;
        match self.iter.peek().ok_or("Fail to parse".to_string())? {
            Token::Less | Token::Greater => unsafe {
                let t = self.iter.next().unwrap_unchecked();
                let rhs = self.parse_term()?;
                Ok(Box::new(Expr::Binary {
                    left: lhs,
                    op: t,
                    right: rhs,
                }))
            },
            _ => Ok(lhs),
        }
    }
    fn parse_term(&mut self) -> Result<Box<Expr>, String> {
        let mut lhs = self.parse_factor()?;
        while let Some(Token::Plus) | Some(Token::Minus) = self.iter.peek() {
            unsafe {
                let t = self.iter.next().unwrap_unchecked();
                let rhs = self.parse_factor()?;
                lhs = Box::new(Expr::Binary {
                    left: lhs,
                    op: t,
                    right: rhs,
                })
            }
        }
        Ok(lhs)
    }

    fn parse_factor(&mut self) -> Result<Box<Expr>, String> {
        let lhs = self.parse_pow()?;
        match self.iter.peek().ok_or("Fail to parse".to_string())? {
            Token::Mult | Token::Div => unsafe {
                let t = self.iter.next().unwrap_unchecked();
                let rhs = self.parse_pow()?;
                Ok(Box::new(Expr::Binary {
                    left: lhs,
                    op: t,
                    right: rhs,
                }))
            },
            _ => Ok(lhs),
        }
    }
    fn parse_pow(&mut self) -> Result<Box<Expr>, String> {
        let lhs = self.parse_unary()?;
        match self.iter.peek().ok_or("Fail to parse".to_string())? {
            Token::Pow => unsafe {
                let t = self.iter.next().unwrap_unchecked();
                let rhs = self.parse_unary()?;
                Ok(Box::new(Expr::Binary {
                    left: lhs,
                    op: t,
                    right: rhs,
                }))
            },
            _ => Ok(lhs),
        }
    }
    fn parse_unary(&mut self) -> Result<Box<Expr>, String> {
        match self.iter.peek().ok_or("Fail to parse".to_string())? {
            Token::Minus | Token::BitFlip => unsafe {
                let t = self.iter.next().unwrap_unchecked();
                let expr = self.parse_unary()?;
                Ok(Box::new(Expr::Unary { op: t, expr: expr }))
            },
            _ => Ok(self.parse_terminal()?),
        }
    }
    fn parse_terminal(&mut self) -> Result<Box<Expr>, String> {
        match self.iter.peek().ok_or("Fail to parse".to_string())? {
            Token::Num(n) => Ok(Box::new(Expr::Num(*n))),
            Token::LeftPar => {
                let expr = self.parse_expression()?;
                self.iter
                    .next_if_eq(&Token::RightPar)
                    .ok_or_else(|| "Missing right parentheses".to_string())?;
                Ok(expr)
            }
            _ => Err(format!("This error message should not even be possible").to_string()),
        }
    }

    fn parse(&mut self) -> Result<Box<Expr>, String> {
        self.parse_expression()
    }
}

fn main() {
    for line in io::stdin().lines() {
        match line.unwrap().as_str() {
            "q" | "quit" => break,
            expr => match Scanner::new(&expr.to_string()) {
                Ok(scanner) => match Parser::new(scanner).parse() {
                    Ok(expr) => println!("{}", expr.interpret()),
                    Err(e) => println!("{e}"),
                },
                Err(e) => println!("{e}"),
            },
        }
    }
}
