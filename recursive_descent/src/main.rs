#![windows_subsystem = "console"]
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
    Ans,
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
            Token::Ans => write!(f, "Ans"),
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
            Some(Token::EOF) => {
                self.add_token(Token::EOF);
                Ok(())
            }
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
            Some('A' | 'a') => Some(Token::Ans),
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
                        if *self.num_buffer.last().unwrap() == b'.'
                            || self.num_buffer.iter().filter(|c| **c == b'.').count() > 1
                        {
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
* comparison -> term (">"| "<") term
* term -> factor ("+"| "-") factor
* factor -> pow ("*" | "/") pow
* pow -> unary ("^") unary
* unary -> ("~" | "-") terminal
* terminal -> Num(n)
*/
struct Parser {
    iter: Peekable<IntoIter<Token>>,
    ans: f64,
}

impl Parser {
    fn new(tokens: Scanner, ans: f64) -> Parser {
        Parser {
            iter: tokens.tokens.into_iter().peekable(),
            ans,
        }
    }

    fn is_end(&mut self) -> bool {
        match self.iter.peek() {
            Some(Token::EOF) => true,
            _ => false,
        }
    }

    fn parse_expression(&mut self) -> Result<Box<Expr>, String> {
        Ok(self.parse_comparison()?)
    }

    fn parse_comparison(&mut self) -> Result<Box<Expr>, String> {
        let mut lhs = self.parse_term()?;
        while let Some(Token::Greater) | Some(Token::Less) | Some(Token::Equal) = self.iter.peek() {
            unsafe {
                let t = self.iter.next().unwrap_unchecked();
                let rhs = self.parse_term()?;
                lhs = Box::new(Expr::Binary {
                    left: lhs,
                    op: t,
                    right: rhs,
                })
            }
        }
        Ok(lhs)
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
        let mut lhs = self.parse_pow()?;
        while let Some(Token::Mult) | Some(Token::Div) = self.iter.peek() {
            unsafe {
                let t = self.iter.next().unwrap_unchecked();
                let rhs = self.parse_pow()?;
                lhs = Box::new(Expr::Binary {
                    left: lhs,
                    op: t,
                    right: rhs,
                })
            }
        }
        Ok(lhs)
    }
    fn parse_pow(&mut self) -> Result<Box<Expr>, String> {
        let lhs = self.parse_unary()?;
        if let Some(Token::Pow) = self.iter.peek() {
            unsafe {
                let t = self.iter.next().unwrap_unchecked();
                let rhs = self.parse_pow()?;
                Ok(Box::new(Expr::Binary {
                    left: lhs,
                    op: t,
                    right: rhs,
                }))
            }
        } else {
            Ok(lhs)
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
            Token::Num(n) => {
                let n = *n;
                self.iter.next();
                Ok(Box::new(Expr::Num(n)))
            }
            Token::LeftPar => {
                self.iter.next();
                let expr = self.parse_expression()?;
                self.iter
                    .next_if_eq(&Token::RightPar)
                    .ok_or_else(|| "Missing right parentheses".to_string())?;
                Ok(expr)
            }
            Token::Ans => {
                self.iter.next();
                Ok(Box::new(Expr::Num(self.ans)))
            }
            _ => Err(format!("Fail to parse").to_string()),
        }
    }

    fn parse(&mut self) -> Result<Box<Expr>, String> {
        let expr = self.parse_expression()?;
        if !self.is_end() {
            return Err(format!(
                "Garbage input at the end, though this should never be called anyways"
            )
            .to_string());
        }
        Ok(expr)
    }
}
fn run(expr: &str, ans: &mut f64, s: &mut String, tree: &mut String) {
    let prev_string = expr;
    match Scanner::new(&expr.to_string()) {
        Ok(scanner) => match Parser::new(scanner, *ans).parse() {
            Ok(expr) => {
                *s = prev_string.to_string();
                *tree = expr.to_string();
                let val = expr.interpret();
                println!("\t\t{}", val);
                *ans = val;
            }
            Err(e) => println!("{e}\n"),
        },
        Err(e) => println!("\t{e}"),
    }
}

fn main() {
    let mut ans: f64 = 0.0;
    let mut s: String = "".to_string();
    let mut tree: String = "".to_string();
    println!("Use 'h' or 'help' to display commands");
    for line in io::stdin().lines() {
        match line.unwrap().as_str().trim() {
            "help" | "h" => println!(
                "Usage: Type expression with + - * / > < = ^ ~ ( ) A a then Enter\nA or a is substituted by previous answer\ntree: Create minimal parse tree of previous expression \nq or quit: Stop the program"
            ),
            "tree" => println!("{tree}"),
            "q" | "quit" => break,
            "" => {
                println!("{s}");
                run(&s.clone(), &mut ans, &mut s, &mut tree)
            }
            expr => run(expr, &mut ans, &mut s, &mut tree),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_expr(input: &str, ans: f64) -> Box<Expr> {
        let s: String = input.to_string();
        let scanner = Scanner::new(&s).unwrap();
        Parser::new(scanner, ans).parse().unwrap()
    }

    #[test]
    fn test_eval_basic() {
        let expr = Expr::Binary {
            left: Box::new(Expr::Num(2.0)),
            op: Token::Plus,
            right: Box::new(Expr::Num(3.0)),
        };
        assert_eq!(5.0, expr.interpret());

        let expr = Expr::Binary {
            left: Box::new(Expr::Num(2.0)),
            op: Token::Minus,
            right: Box::new(Expr::Num(3.0)),
        };
        assert_eq!(-1.0, expr.interpret());

        let expr = Expr::Binary {
            left: Box::new(Expr::Num(2.0)),
            op: Token::Mult,
            right: Box::new(Expr::Num(3.0)),
        };
        assert_eq!(6.0, expr.interpret());
    }

    #[test]
    fn tokenizer() {
        let s = "1.33 + 2.5 * -(3 + 2 -- 3)".to_string();
        let scanner = Scanner::new(&s).unwrap();
        let toks = vec![
            Token::Num(1.33),
            Token::Plus,
            Token::Num(2.5),
            Token::Mult,
            Token::Minus,
            Token::LeftPar,
            Token::Num(3.0),
            Token::Plus,
            Token::Num(2.0),
            Token::Minus,
            Token::Minus,
            Token::Num(3.0),
            Token::RightPar,
            Token::EOF,
        ];

        assert_eq!(scanner.tokens, toks);
    }

    #[test]
    fn test_parse_simple() {
        let ast = parse_expr("1 + 2", 0.0);
        let expected = Box::new(Expr::Binary {
            left: Box::new(Expr::Num(1.0)),
            op: Token::Plus,
            right: Box::new(Expr::Num(2.0)),
        });
        assert_eq!(format!("{}", ast), format!("{}", expected));
    }

    #[test]
    fn test_parse_neg() {
        let ast = parse_expr("1 + -2", 0.0);
        let expected = Box::new(Expr::Binary {
            left: Box::new(Expr::Num(1.0)),
            op: Token::Plus,
            right: Box::new(Expr::Unary {
                op: Token::Minus,
                expr: Box::new(Expr::Num(2.0)),
            }),
        });
        assert_eq!(format!("{}", ast), format!("{}", expected));
    }

    #[test]
    fn test_parse_parentheses() {
        let ast = parse_expr("1 + (2 + 3) - 2", 0.0);
        assert_eq!(ast.interpret(), 4.0);
    }

    #[test]
    fn test_parse_left_associative() {
        let ast = parse_expr("1 - 2 - 3 - 4", 0.0);
        assert_eq!(ast.interpret(), -8.0);
    }

    #[test]
    fn test_parse_invalid_easy() {
        assert!(
            Scanner::new(&"1 + * 2".to_string())
                .ok()
                .and_then(|s| Parser::new(s, 0.0).parse().ok())
                .is_none()
        );
    }

    #[test]
    fn test_parse_invalid_unfinished() {
        assert!(
            Scanner::new(&"1 + (2 * 3) - ".to_string())
                .ok()
                .and_then(|s| Parser::new(s, 0.0).parse().ok())
                .is_none()
        );
    }

    #[test]
    fn test_parse_order() {
        let ast = parse_expr("1 * 2 + 3", 0.0);
        assert_eq!(ast.interpret(), 5.0);
    }

    #[test]
    fn test_parse_order_pow() {
        let ast = parse_expr("2^3^4", 0.0);
        assert_eq!(ast.interpret(), 2f64.powf(3f64.powf(4.0)));
    }

    #[test]
    fn test_invalid_number() {
        assert!(Scanner::new(&"1..2".to_string()).is_err());
    }
}
