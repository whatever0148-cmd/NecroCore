//! Tokenizer and parser for the COBOL subset. Comment lines start with `*` as the
//! first non-blank character. Sentences end with a period.
use necro_core::Span;

#[derive(Clone, Debug, PartialEq)]
pub struct Tok {
    pub text: String,
    pub upper: String,
    pub span: Span,
    pub is_string: bool,
}

pub fn tokenize(src: &str) -> Vec<Tok> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    for (ln, line) in src.split_inclusive('\n').enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('*') || trimmed.starts_with("*>") {
            offset += line.len();
            continue;
        }
        let bytes = line.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() {
            let c = bytes[i] as char;
            if c.is_whitespace() {
                i += 1;
                continue;
            }
            let start = i;
            // A PICTURE string such as 9(5)V99 is one token: take it whole.
            if out.last().map(|t: &Tok| t.upper == "PIC" || t.upper == "PICTURE").unwrap_or(false) {
                while i < bytes.len() && !(bytes[i] as char).is_whitespace() {
                    i += 1;
                }
                let mut end = i;
                if end > start && bytes[end - 1] == b'.' {
                    end -= 1;
                }
                let text = line[start..end].to_string();
                out.push(Tok { upper: text.to_uppercase(), text, span: Span::new(offset + start, offset + end, ln + 1), is_string: false });
                if end < i {
                    out.push(Tok { text: ".".into(), upper: ".".into(), span: Span::new(offset + end, offset + end + 1, ln + 1), is_string: false });
                }
                continue;
            }
            if c == '\'' || c == '"' {
                i += 1;
                while i < bytes.len() && bytes[i] as char != c {
                    i += 1;
                }
                let text = line[start + 1..i.min(bytes.len())].to_string();
                i += 1;
                out.push(Tok { upper: text.to_uppercase(), text, span: Span::new(offset + start, offset + i, ln + 1), is_string: true });
                continue;
            }
            if c == '.' && (i + 1 >= bytes.len() || (bytes[i + 1] as char).is_whitespace()) {
                out.push(Tok { text: ".".into(), upper: ".".into(), span: Span::new(offset + i, offset + i + 1, ln + 1), is_string: false });
                i += 1;
                continue;
            }
            if "()=<>+*/".contains(c) {
                // single-char operators and parens (but keep '-' inside names)
                i += 1;
                let text = c.to_string();
                out.push(Tok { upper: text.clone(), text, span: Span::new(offset + start, offset + i, ln + 1), is_string: false });
                continue;
            }
            while i < bytes.len() {
                let d = bytes[i] as char;
                if d.is_whitespace() || "()=<>+*/".contains(d) {
                    break;
                }
                if d == '.' && (i + 1 >= bytes.len() || (bytes[i + 1] as char).is_whitespace()) {
                    break;
                }
                i += 1;
            }
            let text = line[start..i].to_string();
            out.push(Tok { upper: text.to_uppercase(), text, span: Span::new(offset + start, offset + i, ln + 1), is_string: false });
        }
        offset += line.len();
    }
    out
}

#[derive(Clone, Debug, PartialEq)]
pub struct DataItem {
    pub level: u32,
    pub name: String,
    pub int_digits: u32,
    pub scale: u32,
    pub signed: bool,
    pub alnum_len: Option<u32>,
    pub comp3: bool,
    pub value: Option<String>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CondOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Cond {
    Cmp(Arith, CondOp, Arith),
    And(Box<Cond>, Box<Cond>),
    Or(Box<Cond>, Box<Cond>),
    Not(Box<Cond>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Arith {
    Num(String),
    Var(String),
    Str(String),
    Bin(Box<Arith>, char, Box<Arith>),
    Neg(Box<Arith>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Move { src: Arith, dst: String, span: Span },
    Arith { dst: String, expr: Arith, rounded: bool, span: Span },
    If { cond: Cond, then: Vec<Stmt>, els: Vec<Stmt>, span: Span },
    Perform { first: String, last: Option<String>, times: Option<Arith>, until: Option<Cond>, span: Span },
    GoTo { para: String, span: Span },
    Display { span: Span },
    StopRun { span: Span },
    Exit { span: Span },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Paragraph {
    pub name: String,
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub program_id: String,
    pub data: Vec<DataItem>,
    pub paragraphs: Vec<Paragraph>,
}

struct P {
    t: Vec<Tok>,
    i: usize,
}

impl P {
    fn peek(&self) -> &str {
        self.t.get(self.i).map(|t| t.upper.as_str()).unwrap_or("")
    }
    fn peek_at(&self, k: usize) -> &str {
        self.t.get(self.i + k).map(|t| t.upper.as_str()).unwrap_or("")
    }
    fn at_end(&self) -> bool {
        self.i >= self.t.len()
    }
    fn span(&self) -> Span {
        self.t.get(self.i).map(|t| t.span).unwrap_or_default()
    }
    fn next(&mut self) -> Result<Tok, String> {
        let t = self.t.get(self.i).cloned().ok_or_else(|| "unexpected end of program".to_string())?;
        self.i += 1;
        Ok(t)
    }
    fn expect(&mut self, kw: &str) -> Result<(), String> {
        let t = self.next()?;
        if t.upper == kw {
            Ok(())
        } else {
            Err(format!("line {}: expected {kw}, found {}", t.span.line, t.text))
        }
    }
    fn accept(&mut self, kw: &str) -> bool {
        if self.peek() == kw {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn skip_to_period(&mut self) {
        while !self.at_end() && self.peek() != "." {
            self.i += 1;
        }
        self.accept(".");
    }
}

fn parse_pic(pic: &str) -> Result<(u32, u32, bool, Option<u32>), String> {
    // Expands 9(5)V99, S9(7)V9(2), 99, X(10)
    let mut signed = false;
    let mut chars = pic.chars().peekable();
    let mut expanded = String::new();
    while let Some(c) = chars.next() {
        if c == '(' {
            let mut n = String::new();
            for d in chars.by_ref() {
                if d == ')' {
                    break;
                }
                n.push(d);
            }
            let count: usize = n.trim().parse().map_err(|_| format!("bad PIC repeat in {pic}"))?;
            let last = expanded.pop().ok_or_else(|| format!("bad PIC {pic}"))?;
            for _ in 0..count {
                expanded.push(last);
            }
        } else {
            expanded.push(c);
        }
    }
    if expanded.starts_with('S') {
        signed = true;
        expanded.remove(0);
    }
    if expanded.starts_with('X') {
        return Ok((0, 0, false, Some(expanded.len() as u32)));
    }
    let (ip, fp) = match expanded.split_once('V') {
        Some((a, b)) => (a, b),
        None => (expanded.as_str(), ""),
    };
    if !ip.chars().all(|c| c == '9') || !fp.chars().all(|c| c == '9') {
        return Err(format!("unsupported PIC {pic}"));
    }
    Ok((ip.len() as u32, fp.len() as u32, signed, None))
}

pub fn parse(src: &str) -> Result<Program, String> {
    let toks = tokenize(src);
    let mut p = P { t: toks, i: 0 };
    let mut program_id = String::from("UNKNOWN");
    let mut data = Vec::new();
    // IDENTIFICATION DIVISION
    while !p.at_end() && p.peek() != "DATA" && p.peek() != "PROCEDURE" {
        if p.peek() == "PROGRAM-ID" {
            p.next()?;
            p.accept(".");
            program_id = p.next()?.text;
            p.skip_to_period();
        } else {
            p.next()?;
        }
    }
    if p.accept("DATA") {
        p.expect("DIVISION")?;
        p.accept(".");
        while !p.at_end() && p.peek() != "PROCEDURE" {
            if p.peek() == "WORKING-STORAGE" || p.peek() == "FILE" || p.peek() == "LINKAGE" {
                p.next()?;
                p.expect("SECTION")?;
                p.accept(".");
                continue;
            }
            let level_tok = p.next()?;
            let level: u32 = level_tok.upper.parse().map_err(|_| format!("line {}: expected level number, found {}", level_tok.span.line, level_tok.text))?;
            let name_tok = p.next()?;
            let mut item = DataItem {
                level,
                name: name_tok.upper.clone(),
                int_digits: 0,
                scale: 0,
                signed: false,
                alnum_len: None,
                comp3: false,
                value: None,
                span: Span::new(level_tok.span.byte_start, name_tok.span.byte_end, level_tok.span.line),
            };
            let mut has_pic = false;
            while !p.at_end() && p.peek() != "." {
                match p.peek() {
                    "PIC" | "PICTURE" => {
                        p.next()?;
                        p.accept("IS");
                        let pic = p.next()?;
                        let (i, s, sg, al) = parse_pic(&pic.upper)?;
                        item.int_digits = i;
                        item.scale = s;
                        item.signed = sg;
                        item.alnum_len = al;
                        item.span.byte_end = pic.span.byte_end;
                        has_pic = true;
                    }
                    "COMP-3" | "COMPUTATIONAL-3" | "PACKED-DECIMAL" => {
                        p.next()?;
                        item.comp3 = true;
                    }
                    "USAGE" => {
                        p.next()?;
                        p.accept("IS");
                    }
                    "VALUE" => {
                        p.next()?;
                        p.accept("IS");
                        let v = p.next()?;
                        item.value = Some(if v.is_string { v.text } else { v.upper });
                    }
                    _ => {
                        p.next()?;
                    }
                }
            }
            p.accept(".");
            if has_pic {
                data.push(item);
            }
        }
    }
    p.expect("PROCEDURE")?;
    p.expect("DIVISION")?;
    p.accept(".");
    let mut paragraphs: Vec<Paragraph> = Vec::new();
    while !p.at_end() {
        // paragraph header: NAME .
        let name_tok = p.next()?;
        if p.peek() != "." {
            return Err(format!("line {}: expected paragraph name followed by a period, found {}", name_tok.span.line, name_tok.text));
        }
        p.next()?;
        let mut stmts = Vec::new();
        while !p.at_end() && !(p.peek_at(1) == "." && is_para_header(&p)) {
            let s = parse_sentence(&mut p)?;
            stmts.extend(s);
        }
        paragraphs.push(Paragraph { name: name_tok.upper, stmts, span: name_tok.span });
    }
    Ok(Program { program_id, data, paragraphs })
}

fn is_para_header(p: &P) -> bool {
    // A bare word followed by a period at sentence start, not a statement keyword.
    let w = p.peek();
    !matches!(w, "STOP" | "EXIT" | "MOVE" | "ADD" | "SUBTRACT" | "MULTIPLY" | "DIVIDE" | "COMPUTE" | "IF" | "PERFORM" | "GO" | "DISPLAY" | "ELSE" | "END-IF" | "CONTINUE")
}

/// Parses statements up to and including the terminating period.
fn parse_sentence(p: &mut P) -> Result<Vec<Stmt>, String> {
    let mut out = Vec::new();
    while !p.at_end() && p.peek() != "." {
        out.push(parse_stmt(p)?);
    }
    p.accept(".");
    Ok(out)
}

fn parse_stmt(p: &mut P) -> Result<Stmt, String> {
    let span = p.span();
    let kw = p.next()?;
    match kw.upper.as_str() {
        "MOVE" => {
            let src = parse_operand(p)?;
            p.expect("TO")?;
            let dst = p.next()?.upper;
            Ok(Stmt::Move { src, dst, span: span_to(span, p) })
        }
        "ADD" => {
            let a = parse_operand(p)?;
            p.expect("TO")?;
            let dst = p.next()?.upper;
            let rounded = p.accept("ROUNDED");
            Ok(Stmt::Arith { expr: Arith::Bin(Box::new(Arith::Var(dst.clone())), '+', Box::new(a)), dst, rounded, span: span_to(span, p) })
        }
        "SUBTRACT" => {
            let a = parse_operand(p)?;
            p.expect("FROM")?;
            let dst = p.next()?.upper;
            let rounded = p.accept("ROUNDED");
            Ok(Stmt::Arith { expr: Arith::Bin(Box::new(Arith::Var(dst.clone())), '-', Box::new(a)), dst, rounded, span: span_to(span, p) })
        }
        "MULTIPLY" => {
            let a = parse_operand(p)?;
            p.expect("BY")?;
            let dst = p.next()?.upper;
            let rounded = p.accept("ROUNDED");
            Ok(Stmt::Arith { expr: Arith::Bin(Box::new(Arith::Var(dst.clone())), '*', Box::new(a)), dst, rounded, span: span_to(span, p) })
        }
        "DIVIDE" => {
            let a = parse_operand(p)?;
            if p.accept("INTO") {
                let dst = p.next()?.upper;
                let rounded = p.accept("ROUNDED");
                Ok(Stmt::Arith { expr: Arith::Bin(Box::new(Arith::Var(dst.clone())), '/', Box::new(a)), dst, rounded, span: span_to(span, p) })
            } else {
                p.expect("BY")?;
                let b = parse_operand(p)?;
                p.expect("GIVING")?;
                let dst = p.next()?.upper;
                let rounded = p.accept("ROUNDED");
                Ok(Stmt::Arith { expr: Arith::Bin(Box::new(a), '/', Box::new(b)), dst, rounded, span: span_to(span, p) })
            }
        }
        "COMPUTE" => {
            let dst = p.next()?.upper;
            let rounded = p.accept("ROUNDED");
            p.expect("=")?;
            let expr = parse_arith(p)?;
            Ok(Stmt::Arith { dst, expr, rounded, span: span_to(span, p) })
        }
        "IF" => {
            let cond = parse_cond(p)?;
            p.accept("THEN");
            let mut then = Vec::new();
            while !p.at_end() && !matches!(p.peek(), "ELSE" | "END-IF" | ".") {
                then.push(parse_stmt(p)?);
            }
            let mut els = Vec::new();
            if p.accept("ELSE") {
                while !p.at_end() && !matches!(p.peek(), "END-IF" | ".") {
                    els.push(parse_stmt(p)?);
                }
            }
            p.accept("END-IF");
            Ok(Stmt::If { cond, then, els, span: span_to(span, p) })
        }
        "PERFORM" => {
            let first = p.next()?.upper;
            let mut last = None;
            if p.accept("THRU") || p.accept("THROUGH") {
                last = Some(p.next()?.upper);
            }
            let mut times = None;
            let mut until = None;
            if p.peek() == "UNTIL" {
                p.next()?;
                until = Some(parse_cond(p)?);
            } else if p.peek_at(1) == "TIMES" {
                times = Some(parse_operand(p)?);
                p.expect("TIMES")?;
            }
            Ok(Stmt::Perform { first, last, times, until, span: span_to(span, p) })
        }
        "GO" => {
            p.accept("TO");
            let para = p.next()?.upper;
            Ok(Stmt::GoTo { para, span: span_to(span, p) })
        }
        "DISPLAY" => {
            while !p.at_end() && !is_stmt_start(p.peek()) && p.peek() != "." && p.peek() != "ELSE" && p.peek() != "END-IF" {
                p.next()?;
            }
            Ok(Stmt::Display { span: span_to(span, p) })
        }
        "STOP" => {
            p.expect("RUN")?;
            Ok(Stmt::StopRun { span: span_to(span, p) })
        }
        "EXIT" | "CONTINUE" => Ok(Stmt::Exit { span: span_to(span, p) }),
        other => Err(format!("line {}: unsupported statement {other}", kw.span.line)),
    }
}

fn is_stmt_start(w: &str) -> bool {
    matches!(w, "MOVE" | "ADD" | "SUBTRACT" | "MULTIPLY" | "DIVIDE" | "COMPUTE" | "IF" | "PERFORM" | "GO" | "DISPLAY" | "STOP" | "EXIT" | "CONTINUE")
}

fn span_to(start: Span, p: &P) -> Span {
    let end = p.t.get(p.i.saturating_sub(1)).map(|t| t.span.byte_end).unwrap_or(start.byte_end);
    Span::new(start.byte_start, end, start.line)
}

fn parse_operand(p: &mut P) -> Result<Arith, String> {
    let t = p.next()?;
    if t.is_string {
        return Ok(Arith::Str(t.text));
    }
    if t.upper == "ZERO" || t.upper == "ZEROS" || t.upper == "ZEROES" {
        return Ok(Arith::Num("0".into()));
    }
    if t.upper == "SPACE" || t.upper == "SPACES" {
        return Ok(Arith::Str(" ".into()));
    }
    if t.upper.chars().next().map(|c| c.is_ascii_digit() || c == '-' && t.upper.len() > 1 && t.upper.as_bytes()[1].is_ascii_digit()).unwrap_or(false) {
        return Ok(Arith::Num(t.upper));
    }
    Ok(Arith::Var(t.upper))
}

fn parse_arith(p: &mut P) -> Result<Arith, String> {
    let mut l = parse_term(p)?;
    while p.peek() == "+" || p.peek() == "-" {
        let op = p.next()?.upper.chars().next().unwrap();
        let r = parse_term(p)?;
        l = Arith::Bin(Box::new(l), op, Box::new(r));
    }
    Ok(l)
}

fn parse_term(p: &mut P) -> Result<Arith, String> {
    let mut l = parse_factor(p)?;
    while p.peek() == "*" || p.peek() == "/" {
        let op = p.next()?.upper.chars().next().unwrap();
        let r = parse_factor(p)?;
        l = Arith::Bin(Box::new(l), op, Box::new(r));
    }
    Ok(l)
}

fn parse_factor(p: &mut P) -> Result<Arith, String> {
    if p.accept("(") {
        let e = parse_arith(p)?;
        p.expect(")")?;
        return Ok(e);
    }
    if p.accept("-") {
        return Ok(Arith::Neg(Box::new(parse_factor(p)?)));
    }
    parse_operand(p)
}

fn parse_cond(p: &mut P) -> Result<Cond, String> {
    let mut l = parse_cond_term(p)?;
    while p.accept("OR") {
        let r = parse_cond_term(p)?;
        l = Cond::Or(Box::new(l), Box::new(r));
    }
    Ok(l)
}

fn parse_cond_term(p: &mut P) -> Result<Cond, String> {
    let mut l = parse_cond_atom(p)?;
    while p.accept("AND") {
        let r = parse_cond_atom(p)?;
        l = Cond::And(Box::new(l), Box::new(r));
    }
    Ok(l)
}

fn parse_cond_atom(p: &mut P) -> Result<Cond, String> {
    if p.accept("(") {
        let c = parse_cond(p)?;
        p.expect(")")?;
        return Ok(c);
    }
    if p.accept("NOT") {
        return Ok(Cond::Not(Box::new(parse_cond_atom(p)?)));
    }
    let l = parse_arith(p)?;
    p.accept("IS");
    let negate = p.accept("NOT");
    let t = p.next()?;
    let op = match t.upper.as_str() {
        "=" | "EQUAL" | "EQUALS" => {
            p.accept("TO");
            CondOp::Eq
        }
        ">" | "GREATER" => {
            p.accept("THAN");
            if p.accept("OR") {
                p.expect("EQUAL")?;
                p.accept("TO");
                CondOp::Ge
            } else {
                CondOp::Gt
            }
        }
        "<" | "LESS" => {
            p.accept("THAN");
            if p.accept("OR") {
                p.expect("EQUAL")?;
                p.accept("TO");
                CondOp::Le
            } else {
                CondOp::Lt
            }
        }
        ">=" => CondOp::Ge,
        "<=" => CondOp::Le,
        other => return Err(format!("line {}: unsupported comparison {other}", t.span.line)),
    };
    // handle ">" "=" written as two tokens
    let op = match (op, p.peek()) {
        (CondOp::Gt, "=") => {
            p.next()?;
            CondOp::Ge
        }
        (CondOp::Lt, "=") => {
            p.next()?;
            CondOp::Le
        }
        (o, _) => o,
    };
    let r = parse_arith(p)?;
    let c = Cond::Cmp(l, op, r);
    Ok(if negate { Cond::Not(Box::new(c)) } else { c })
}
