#![windows_subsystem = "console"]
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::iter::Peekable;
use std::str::{Chars, from_utf8_unchecked};
use std::{fmt, io};

//the other option is making Num into its own object to make it hashable but that is not very
//robust (well, the code is not robust anyways, but still)
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

#[derive(PartialEq, Debug, Clone, Copy, Eq, Hash)]
enum Terminal {
    Num,
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

impl Token {
    fn to_term(&self) -> Terminal {
        match self {
            Token::Num(_) => Terminal::Num,
            Token::RightPar => Terminal::RightPar,
            Token::LeftPar => Terminal::LeftPar,
            Token::Plus => Terminal::Plus,
            Token::Minus => Terminal::Minus,
            Token::Mult => Terminal::Mult,
            Token::Div => Terminal::Div,
            Token::Pow => Terminal::Pow,
            Token::Equal => Terminal::Equal,
            Token::Greater => Terminal::Greater,
            Token::Less => Terminal::Less,
            Token::BitFlip => Terminal::BitFlip,
            Token::EOF => Terminal::EOF,
            Token::Ans => Terminal::Ans,
        }
    }
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
    fn to_expr(node: &Node) -> Expr {
        match node {
            Node::Terminal(tok) => match tok {
                Token::Num(n) => Expr::Num(*n),
                _ => panic!("???"),
            },
            Node::NonTerm(lhs, children) => match lhs {
                NonTerm::Unary => {
                    if children.len() == 2 {
                        let op = match &children[0] {
                            Node::Terminal(t) => *t,
                            _ => panic!("???"),
                        };
                        let expr = Box::new(Self::to_expr(&children[1]));
                        Expr::Unary { op, expr }
                    } else {
                        Self::to_expr(&children[0])
                    }
                }
                NonTerm::Term
                | NonTerm::Factor
                | NonTerm::Pow
                | NonTerm::Comparison
                | NonTerm::Expr => {
                    if children.len() == 3 {
                        let left = Box::new(Self::to_expr(&children[0]));
                        let op = match &children[1] {
                            Node::Terminal(t) => *t,
                            _ => panic!("???"),
                        };
                        let right = Box::new(Self::to_expr(&children[2]));
                        Expr::Binary { left, op, right }
                    } else if children.len() == 1 {
                        Self::to_expr(&children[0])
                    } else {
                        panic!("???")
                    }
                }
                NonTerm::Terminal => Self::to_expr(&children[0]),
            },
        }
    }

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
            Token::Num(n) => write!(f, "{n}"),
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum NonTerm {
    Expr,
    Comparison,
    Term,
    Factor,
    Pow,
    Unary,
    Terminal,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
enum Symbol {
    Terminal(Terminal),
    NonTerm(NonTerm),
}

#[derive(Debug, Clone)]
struct EarleyItem {
    id: usize,
    rule_index: usize,
    matched: usize,
    remains: usize,
    start: usize,
    children: Vec<Vec<(usize, usize)>>,
}

impl PartialEq for EarleyItem {
    fn eq(&self, other: &Self) -> bool {
        self.rule_index == other.rule_index
            && self.start == other.start
            && self.matched == other.matched
        //&& self.children == other.children
    }
}

impl Eq for EarleyItem {}

impl Hash for EarleyItem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rule_index.hash(state);
        self.start.hash(state);
        self.matched.hash(state);
        //self.children.hash(state);
    }
}

struct Recogniser<'a> {
    s: Vec<HashSet<EarleyItem>>,
    g: Grammar,
    token: &'a [Token],
    counter: usize,
}

#[derive(Debug)]
struct Rule {
    lhs: NonTerm,
    rhs: Vec<Symbol>,
    length: usize,
}

struct Grammar {
    rules: Vec<Rule>,
}

#[derive(Debug, Clone)]
enum Node {
    NonTerm(NonTerm, Vec<Node>),
    Terminal(Token),
}

impl<'a> Recogniser<'a> {
    fn new(g: Grammar, scanner: &'a Scanner) -> Recogniser<'a> {
        Recogniser {
            s: vec![HashSet::new(); scanner.tokens.len() + 1],
            g,
            token: &scanner.tokens,
            counter: 0,
        }
    }

    fn counter(&mut self) -> usize {
        let num = self.counter;
        self.counter += 1;
        num
    }

    fn counter_revert(&mut self) {
        self.counter -= 1;
    }

    fn handle(&mut self, index: usize) {
        let mut queue: VecDeque<EarleyItem> = self.s.get(index).unwrap().iter().cloned().collect();
        while let Some(item) = queue.pop_front() {
            if item.remains <= 0 {
                self.complete(&item, index, &mut queue);
                continue;
            }
            let s: Symbol = self
                .g
                .rules
                .get(item.rule_index)
                .unwrap()
                .rhs
                .get(item.matched)
                .unwrap()
                .clone();
            match s {
                Symbol::NonTerm(n) => self.predicts(&n, index, &mut queue),
                Symbol::Terminal(n) if index < self.token.len() => {
                    self.scan(&item, &n, index, &mut queue)
                }
                _ => (),
            }
        }
    }

    fn predicts(&mut self, s: &NonTerm, index: usize, queue: &mut VecDeque<EarleyItem>) {
        let rules = std::mem::take(&mut self.g.rules);
        for (i, rule) in rules.iter().enumerate() {
            if *s == rule.lhs {
                let e: EarleyItem = EarleyItem {
                    id: self.counter(),
                    rule_index: i,
                    matched: 0,
                    remains: rule.length,
                    start: index,
                    children: vec![vec![]; rule.length],
                };
                if self.s.get_mut(index).unwrap().insert(e.clone()) {
                    queue.push_back(e.clone());
                } else {
                    self.counter_revert();
                }
            }
        }
        self.g.rules = rules;
    }

    fn scan(
        &mut self,
        e: &EarleyItem,
        s: &Terminal,
        index: usize,
        queue: &mut VecDeque<EarleyItem>,
    ) {
        if self.token.get(index).unwrap().to_term() == *s {
            let mut new_e: EarleyItem = EarleyItem {
                id: self.counter(),
                rule_index: e.rule_index,
                matched: e.matched + 1,
                remains: e.remains - 1,
                start: e.start,
                children: e.children.clone(),
            };
            new_e.children[new_e.matched - 1].push((index, new_e.id));
            if self.s.get_mut(index + 1).unwrap().insert(new_e.clone()) {
                queue.push_back(new_e);
            } else {
                self.counter_revert();
            }
        }
    }

    fn complete(&mut self, e: &EarleyItem, index: usize, queue: &mut VecDeque<EarleyItem>) {
        let hs: HashSet<EarleyItem> = self.s.get(e.start).unwrap().clone();
        for item in hs {
            match self.next_symbol(&item) {
                Some(Symbol::NonTerm(n)) if *n == self.g.rules.get(e.rule_index).unwrap().lhs => {
                    let mut new_e: EarleyItem = EarleyItem {
                        id: self.counter(),
                        rule_index: item.rule_index,
                        matched: item.matched + 1,
                        remains: item.remains - 1,
                        start: item.start,
                        children: item.children.clone(),
                    };
                    new_e.children[new_e.matched - 1].push((index, e.id));
                    if self.s.get_mut(index).unwrap().insert(new_e.clone()) {
                        queue.push_back(new_e);
                    } else {
                        self.counter_revert();
                    }
                }
                _ => (),
            }
        }
    }

    fn next_symbol(&self, e: &EarleyItem) -> Option<&Symbol> {
        self.g.rules.get(e.rule_index).unwrap().rhs.get(e.matched)
    }

    fn recognise(&mut self, start_rule: usize) -> bool {
        let rule_length = self.g.rules.get(start_rule).unwrap().length;
        let mut start = EarleyItem {
            id: self.counter(),
            rule_index: start_rule,
            matched: 0,
            remains: rule_length,
            start: 0,
            children: vec![vec![]; rule_length],
        };
        self.s.get_mut(0).unwrap().insert(start.clone());
        for i in 0..=self.s.len() - 1 {
            self.handle(i);
        }
        let hs: &HashSet<EarleyItem> = self.s.get(self.s.len() - 1).unwrap();
        start.matched = start.remains;
        start.remains = 0;
        for item in hs {
            if *item == start {
                return true;
            }
        }
        false
    }

    fn clean(&mut self) {
        self.s = vec![];
    }
}

struct Parser;

impl Parser {
    fn make_map(r: &Recogniser) -> HashMap<usize, EarleyItem> {
        let mut map = HashMap::new();
        for set in &r.s {
            for item in set {
                map.insert(item.id, item.clone());
            }
        }
        map
    }

    fn build_tree_singular(id: usize, r: &Recogniser, map: &HashMap<usize, EarleyItem>) -> Node {
        let item = &map[&id];
        let rule = &r.g.rules[item.rule_index];
        let mut tree: Vec<Node> = Vec::new();
        for (i, rhs_item) in item.children.iter().enumerate() {
            if let Some(&(index, child_id)) = rhs_item.first() {
                match &rule.rhs[i] {
                    Symbol::Terminal(_) => {
                        tree.push(Node::Terminal(r.token[index].clone()));
                    }
                    Symbol::NonTerm(_) => {
                        tree.push(Self::build_tree_singular(child_id, r, map));
                    }
                }
            }
        }
        Node::NonTerm(rule.lhs.clone(), tree)
    }

    //This part here does not work at all because i used a hashset which actually get rids of
    //all unambiguos grammar and keep only one interpretation, I think? maybe? idk
    fn build_trees(id: usize, r: &Recogniser, map: &HashMap<usize, EarleyItem>) -> Vec<Node> {
        let item = &map[&id];
        let rule = &r.g.rules[item.rule_index];
        let mut all_trees: Vec<Vec<Node>> = Vec::new();
        for (i, rhs_item) in item.children.iter().enumerate() {
            let mut trees = Vec::new();
            for &(index, child_id) in rhs_item {
                match &rule.rhs[i] {
                    Symbol::Terminal(_) => {
                        trees.push(Node::Terminal(r.token[index].clone()));
                    }
                    Symbol::NonTerm(_) => {
                        trees.extend(Self::build_trees(child_id, r, map));
                    }
                }
            }
            all_trees.push(trees);
        }
        Self::cartesian(rule.lhs.clone(), &all_trees)
    }

    fn cartesian(lhs: NonTerm, rhs_items: &Vec<Vec<Node>>) -> Vec<Node> {
        if rhs_items.is_empty() {
            return vec![Node::NonTerm(lhs, vec![])];
        }
        let mut results: Vec<Vec<Node>> = rhs_items[0].iter().map(|x| vec![x.clone()]).collect();
        for rhs_item in rhs_items.iter().skip(1) {
            let mut added_results: Vec<Vec<Node>> = Vec::new();
            for partial in &results {
                for node in rhs_item {
                    let mut partial_clone = partial.clone();
                    partial_clone.push(node.clone());
                    added_results.push(partial_clone);
                }
            }
            results = added_results;
        }
        results
            .into_iter()
            .map(|x| Node::NonTerm(lhs.clone(), x))
            .collect()
    }
}

/*
* Grammar:
* expr -> comparison
*
* comparison -> comparison ">" term
* comparison -> comparison "<" term
* comparison -> term
*
* term -> term "+" factor
* term -> term "-" factor
* term -> factor
*
* factor -> factor "*" pow
* factor -> factor "/" pow
* factor -> pow
*
* pow -> unary "^" pow
* pow -> unary
*
* unary -> "~" unary
* unary -> "-" unary
* unary -> terminal
*
* terminal -> Num
* terminal -> Ans
* terminal -> "(" expr ")"
*/

fn init_grammar() -> Grammar {
    Grammar {
        rules: vec![
            // expr -> comparison
            Rule {
                lhs: NonTerm::Expr,
                rhs: vec![Symbol::NonTerm(NonTerm::Comparison)],
                length: 1,
            },
            // comparison -> comparison ">" term
            Rule {
                lhs: NonTerm::Comparison,
                rhs: vec![
                    Symbol::NonTerm(NonTerm::Comparison),
                    Symbol::Terminal(Terminal::Greater),
                    Symbol::NonTerm(NonTerm::Term),
                ],
                length: 3,
            },
            // comparison -> comparison "<" term
            Rule {
                lhs: NonTerm::Comparison,
                rhs: vec![
                    Symbol::NonTerm(NonTerm::Comparison),
                    Symbol::Terminal(Terminal::Less),
                    Symbol::NonTerm(NonTerm::Term),
                ],
                length: 3,
            },
            // comparison -> term
            Rule {
                lhs: NonTerm::Comparison,
                rhs: vec![Symbol::NonTerm(NonTerm::Term)],
                length: 1,
            },
            // term -> term "+" factor
            Rule {
                lhs: NonTerm::Term,
                rhs: vec![
                    Symbol::NonTerm(NonTerm::Term),
                    Symbol::Terminal(Terminal::Plus),
                    Symbol::NonTerm(NonTerm::Factor),
                ],
                length: 3,
            },
            // term -> term "-" factor
            Rule {
                lhs: NonTerm::Term,
                rhs: vec![
                    Symbol::NonTerm(NonTerm::Term),
                    Symbol::Terminal(Terminal::Minus),
                    Symbol::NonTerm(NonTerm::Factor),
                ],
                length: 3,
            },
            // term -> factor
            Rule {
                lhs: NonTerm::Term,
                rhs: vec![Symbol::NonTerm(NonTerm::Factor)],
                length: 1,
            },
            // factor -> factor "*" pow
            Rule {
                lhs: NonTerm::Factor,
                rhs: vec![
                    Symbol::NonTerm(NonTerm::Factor),
                    Symbol::Terminal(Terminal::Mult),
                    Symbol::NonTerm(NonTerm::Pow),
                ],
                length: 3,
            },
            // factor -> factor "/" pow
            Rule {
                lhs: NonTerm::Factor,
                rhs: vec![
                    Symbol::NonTerm(NonTerm::Factor),
                    Symbol::Terminal(Terminal::Div),
                    Symbol::NonTerm(NonTerm::Pow),
                ],
                length: 3,
            },
            // factor -> pow
            Rule {
                lhs: NonTerm::Factor,
                rhs: vec![Symbol::NonTerm(NonTerm::Pow)],
                length: 1,
            },
            // pow -> unary "^" pow   (right associative)
            Rule {
                lhs: NonTerm::Pow,
                rhs: vec![
                    Symbol::NonTerm(NonTerm::Unary),
                    Symbol::Terminal(Terminal::Pow),
                    Symbol::NonTerm(NonTerm::Pow),
                ],
                length: 3,
            },
            // pow -> unary
            Rule {
                lhs: NonTerm::Pow,
                rhs: vec![Symbol::NonTerm(NonTerm::Unary)],
                length: 1,
            },
            // unary -> "~" unary
            Rule {
                lhs: NonTerm::Unary,
                rhs: vec![
                    Symbol::Terminal(Terminal::BitFlip),
                    Symbol::NonTerm(NonTerm::Unary),
                ],
                length: 2,
            },
            // unary -> "-" unary
            Rule {
                lhs: NonTerm::Unary,
                rhs: vec![
                    Symbol::Terminal(Terminal::Minus),
                    Symbol::NonTerm(NonTerm::Unary),
                ],
                length: 2,
            },
            // unary -> terminal
            Rule {
                lhs: NonTerm::Unary,
                rhs: vec![Symbol::NonTerm(NonTerm::Terminal)],
                length: 1,
            },
            // terminal -> Num
            Rule {
                lhs: NonTerm::Terminal,
                rhs: vec![Symbol::Terminal(Terminal::Num)],
                length: 1,
            },
            // terminal -> Ans
            Rule {
                lhs: NonTerm::Terminal,
                rhs: vec![Symbol::Terminal(Terminal::Ans)],
                length: 1,
            },
            // terminal -> "(" expr ")"
            Rule {
                lhs: NonTerm::Terminal,
                rhs: vec![
                    Symbol::Terminal(Terminal::LeftPar),
                    Symbol::NonTerm(NonTerm::Expr),
                    Symbol::Terminal(Terminal::RightPar),
                ],
                length: 3,
            },
        ],
    }
}
fn start_id(r: &Recogniser, rule_index: usize) -> usize {
    r.s[r.s.len() - 1]
        .iter()
        .find(|x| x.rule_index == rule_index)
        .unwrap()
        .id
}

fn start_id_vecs(r: &Recogniser, rule_index: usize) -> Vec<usize> {
    r.s[r.s.len() - 1]
        .iter()
        .filter(|x| x.rule_index == rule_index)
        .map(|x| x.id)
        .collect()
}

fn main() {
    println!("Type q or quit to Quit; otherwise just type some math expression");
    for line in io::stdin().lines() {
        let line = line.unwrap().to_string();
        if line == "q" || line == "quit" {
            break;
        }
        match Scanner::new(&line) {
            Ok(scanner) => {
                let mut rec = Recogniser::new(init_grammar(), &scanner);
                let result = rec.recognise(0);
                if result == true {
                    println!("\tYep that is definitely a math expression!");
                    let map = &Parser::make_map(&rec);
                    let start_ids = start_id_vecs(&rec, 0);
                    for start_id in start_ids {
                        let node: Node = Parser::build_tree_singular(start_id, &rec, &map);
                        println!("{:#?}", node);
                        println!("{}", Expr::to_expr(&node).interpret());
                    }
                } else {
                    println!("\tI do not recognise what that is");
                }
            }
            Err(_) => println!("\tI do not recognise what that is"),
        }
    }
}
