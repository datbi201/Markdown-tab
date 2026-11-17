//! Cong thuc toan: phan tich LaTeX (tap con thong dung) -> cay `Node`,
//! ve truc tiep bang egui (phan so, can, chi so tren/duoi, ma tran, ngoac co gian...),
//! xuat OMML (Word) va MathML (HTML).

use egui::{
    epaint::Shape,
    text::{LayoutJob, TextFormat},
    Color32, FontFamily, FontId, Pos2, Stroke, Vec2,
};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

// =============================================================== cay cong thuc
#[derive(Clone, Debug, PartialEq)]
pub enum Accent {
    Hat,
    Bar,
    Vec,
    Dot,
    DDot,
    Tilde,
    Underline,
    OverBrace,
    UnderBrace,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Sty {
    pub bold: bool,
    pub roman: bool,
}

#[derive(Clone, Debug)]
pub enum Node {
    Ident(String),          // bien (nghieng)
    Num(String),            // so
    Op(String, bool),       // toan tu; true = quan he (=, <, ...), false = toan tu hai ngoi (+, -...)
    Sym(String),            // ky hieu thang (dau cau, vo cuc...)
    Text(String),           // chu thang (\text)
    Func(String),           // ten ham: sin, log, lim...
    BigOp(String),          // ∑ ∏ ∫ ...
    Row(Vec<Node>),
    Newline,
    Script { base: Box<Node>, sup: Option<Box<Node>>, sub: Option<Box<Node>> },
    Frac(Box<Node>, Box<Node>, bool), // tu, mau, co vach ngang
    Sqrt(Option<Box<Node>>, Box<Node>),
    Delim(String, Box<Node>, String),
    Matrix { rows: Vec<Vec<Node>>, open: String, close: String, align: Vec<char> },
    Accent(Accent, Box<Node>),
    Over(Box<Node>, Box<Node>),  // base, tren
    Under(Box<Node>, Box<Node>), // base, duoi
    Styled(Sty, Box<Node>),
    Space(f32),
}

pub struct Parsed {
    pub root: Node,
    pub errors: Vec<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum K {
    Id,
    Sym,
    Bin,
    Rel,
    Big,
}

const SYMS: &[(&str, &str, K)] = &[
    // chu Hy Lap thuong
    ("alpha", "α", K::Id), ("beta", "β", K::Id), ("gamma", "γ", K::Id), ("delta", "δ", K::Id),
    ("epsilon", "ϵ", K::Id), ("varepsilon", "ε", K::Id), ("zeta", "ζ", K::Id), ("eta", "η", K::Id),
    ("theta", "θ", K::Id), ("vartheta", "ϑ", K::Id), ("iota", "ι", K::Id), ("kappa", "κ", K::Id),
    ("lambda", "λ", K::Id), ("mu", "μ", K::Id), ("nu", "ν", K::Id), ("xi", "ξ", K::Id),
    ("omicron", "ο", K::Id), ("pi", "π", K::Id), ("varpi", "ϖ", K::Id), ("rho", "ρ", K::Id),
    ("varrho", "ϱ", K::Id), ("sigma", "σ", K::Id), ("varsigma", "ς", K::Id), ("tau", "τ", K::Id),
    ("upsilon", "υ", K::Id), ("phi", "ϕ", K::Id), ("varphi", "φ", K::Id), ("chi", "χ", K::Id),
    ("psi", "ψ", K::Id), ("omega", "ω", K::Id),
    // chu Hy Lap hoa
    ("Gamma", "Γ", K::Sym), ("Delta", "Δ", K::Sym), ("Theta", "Θ", K::Sym), ("Lambda", "Λ", K::Sym),
    ("Xi", "Ξ", K::Sym), ("Pi", "Π", K::Sym), ("Sigma", "Σ", K::Sym), ("Upsilon", "Υ", K::Sym),
    ("Phi", "Φ", K::Sym), ("Psi", "Ψ", K::Sym), ("Omega", "Ω", K::Sym),
    // toan tu hai ngoi
    ("pm", "±", K::Bin), ("mp", "∓", K::Bin), ("times", "×", K::Bin), ("div", "÷", K::Bin),
    ("cdot", "⋅", K::Bin), ("ast", "∗", K::Bin), ("star", "⋆", K::Bin), ("circ", "∘", K::Bin),
    ("bullet", "∙", K::Bin), ("oplus", "⊕", K::Bin), ("ominus", "⊖", K::Bin), ("otimes", "⊗", K::Bin),
    ("odot", "⊙", K::Bin), ("cap", "∩", K::Bin), ("cup", "∪", K::Bin), ("wedge", "∧", K::Bin),
    ("land", "∧", K::Bin), ("vee", "∨", K::Bin), ("lor", "∨", K::Bin), ("setminus", "∖", K::Bin),
    // quan he
    ("le", "≤", K::Rel), ("leq", "≤", K::Rel), ("ge", "≥", K::Rel), ("geq", "≥", K::Rel),
    ("ne", "≠", K::Rel), ("neq", "≠", K::Rel), ("approx", "≈", K::Rel), ("sim", "∼", K::Rel),
    ("simeq", "≃", K::Rel), ("cong", "≅", K::Rel), ("equiv", "≡", K::Rel), ("propto", "∝", K::Rel),
    ("ll", "≪", K::Rel), ("gg", "≫", K::Rel), ("in", "∈", K::Rel), ("notin", "∉", K::Rel),
    ("ni", "∋", K::Rel), ("subset", "⊂", K::Rel), ("supset", "⊃", K::Rel), ("subseteq", "⊆", K::Rel),
    ("supseteq", "⊇", K::Rel), ("perp", "⊥", K::Rel), ("parallel", "∥", K::Rel), ("mid", "∣", K::Rel),
    ("to", "→", K::Rel), ("rightarrow", "→", K::Rel), ("leftarrow", "←", K::Rel),
    ("Rightarrow", "⇒", K::Rel), ("Leftarrow", "⇐", K::Rel), ("leftrightarrow", "↔", K::Rel),
    ("Leftrightarrow", "⇔", K::Rel), ("iff", "⇔", K::Rel), ("implies", "⇒", K::Rel),
    ("mapsto", "↦", K::Rel), ("longrightarrow", "⟶", K::Rel), ("longleftarrow", "⟵", K::Rel),
    ("Longrightarrow", "⟹", K::Rel), ("uparrow", "↑", K::Sym), ("downarrow", "↓", K::Sym),
    ("rightleftharpoons", "⇌", K::Rel), ("doteq", "≐", K::Rel), ("asymp", "≍", K::Rel),
    // khac
    ("infty", "∞", K::Sym), ("partial", "∂", K::Sym), ("nabla", "∇", K::Sym), ("forall", "∀", K::Sym),
    ("exists", "∃", K::Sym), ("neg", "¬", K::Sym), ("lnot", "¬", K::Sym), ("emptyset", "∅", K::Sym),
    ("varnothing", "∅", K::Sym), ("ldots", "…", K::Sym), ("dots", "…", K::Sym), ("cdots", "⋯", K::Sym),
    ("vdots", "⋮", K::Sym), ("ddots", "⋱", K::Sym), ("angle", "∠", K::Sym), ("triangle", "△", K::Sym),
    ("degree", "°", K::Sym), ("prime", "′", K::Sym), ("hbar", "ℏ", K::Id), ("ell", "ℓ", K::Id),
    ("Re", "ℜ", K::Sym), ("Im", "ℑ", K::Sym), ("aleph", "ℵ", K::Sym), ("wp", "℘", K::Sym),
    ("square", "□", K::Sym), ("therefore", "∴", K::Sym), ("because", "∵", K::Sym),
    ("checkmark", "✓", K::Sym), ("dagger", "†", K::Sym), ("S", "§", K::Sym), ("P", "¶", K::Sym),
    ("%", "%", K::Sym), ("$", "$", K::Sym), ("#", "#", K::Sym), ("&", "&", K::Sym), ("_", "_", K::Sym),
    ("{", "{", K::Sym), ("}", "}", K::Sym), ("|", "‖", K::Sym),
    // toan tu lon
    ("sum", "∑", K::Big), ("prod", "∏", K::Big), ("coprod", "∐", K::Big), ("int", "∫", K::Big),
    ("iint", "∬", K::Big), ("iiint", "∭", K::Big), ("oint", "∮", K::Big), ("bigcup", "⋃", K::Big),
    ("bigcap", "⋂", K::Big), ("bigoplus", "⨁", K::Big), ("bigotimes", "⨂", K::Big),
    ("bigvee", "⋁", K::Big), ("bigwedge", "⋀", K::Big),
];

const FUNCS: &[&str] = &[
    "sin", "cos", "tan", "cot", "sec", "csc", "arcsin", "arccos", "arctan", "sinh", "cosh", "tanh",
    "coth", "log", "ln", "lg", "exp", "det", "dim", "ker", "deg", "gcd", "hom", "arg", "min", "max",
    "sup", "inf", "lim", "limsup", "liminf", "Pr", "mod", "rank", "tr", "sgn", "lcm",
];

fn lookup_sym(name: &str) -> Option<(&'static str, K)> {
    SYMS.iter().find(|(n, _, _)| *n == name).map(|(_, u, k)| (*u, *k))
}

fn is_limit_func(name: &str) -> bool {
    matches!(name, "lim" | "limsup" | "liminf" | "max" | "min" | "sup" | "inf" | "det" | "gcd" | "Pr")
}

fn is_limits_bigop(s: &str) -> bool {
    !matches!(s, "∫" | "∬" | "∭" | "∮")
}

fn blackboard(c: char) -> Option<&'static str> {
    Some(match c {
        'R' => "ℝ", 'N' => "ℕ", 'Z' => "ℤ", 'Q' => "ℚ", 'C' => "ℂ", 'H' => "ℍ", 'P' => "ℙ",
        _ => return None,
    })
}

fn classify_char(c: char) -> Node {
    let s = c.to_string();
    match c {
        '+' | '-' | '\u{2212}' | '±' | '∓' | '×' | '÷' | '⋅' | '·' | '∙' | '∗' | '*' | '/' | '∪' | '∩' | '∧' | '∨'
        | '⊕' | '⊗' | '∘' => Node::Op(if c == '\u{2212}' { "−".into() } else if c == '-' { "−".into() } else if c == '*' { "∗".into() } else { s }, false),
        '=' | '<' | '>' | '≤' | '≥' | '≠' | '≈' | '∼' | '≅' | '≡' | '∝' | '≪' | '≫' | '∈' | '∉' | '∋' | '⊂'
        | '⊃' | '⊆' | '⊇' | '→' | '←' | '↔' | '⇒' | '⇐' | '⇔' | '↦' | '⟶' | '∥' | '⊥' => Node::Op(s, true),
        '∑' | '∏' | '∫' | '∬' | '∭' | '∮' | '⋃' | '⋂' => Node::BigOp(s),
        c if c.is_ascii_digit() => Node::Num(s),
        c if ('α'..='ω').contains(&c) || matches!(c, 'ϵ' | 'ϑ' | 'ϖ' | 'ϱ' | 'ϕ' | 'ℏ' | 'ℓ') => Node::Ident(s),
        c if c.is_alphabetic() => Node::Ident(s),
        _ => Node::Sym(s),
    }
}

// ==================================================================== parser
#[derive(Clone, Copy, PartialEq)]
enum Stop {
    Top,
    Brace,
    Right,
    Cell,
}

struct P {
    s: Vec<char>,
    i: usize,
    errors: Vec<String>,
    stops: Vec<Stop>,
    depth: usize,
}

impl P {
    fn peek(&self) -> Option<char> {
        self.s.get(self.i).copied()
    }
    fn starts(&self, pat: &str) -> bool {
        let mut j = self.i;
        for c in pat.chars() {
            if self.s.get(j).copied() != Some(c) {
                return false;
            }
            j += 1;
        }
        true
    }
    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.i += 1;
        }
    }
    fn at_stop(&self) -> bool {
        match self.stops.last() {
            Some(Stop::Brace) => self.peek() == Some('}'),
            Some(Stop::Right) => self.starts("\\right"),
            Some(Stop::Cell) => self.peek() == Some('&') || self.starts("\\\\") || self.starts("\\end"),
            _ => false,
        }
    }
    fn read_name(&mut self) -> String {
        // sau dau '\\'
        let mut name = String::new();
        match self.peek() {
            Some(c) if c.is_ascii_alphabetic() => {
                while let Some(c) = self.peek() {
                    if c.is_ascii_alphabetic() {
                        name.push(c);
                        self.i += 1;
                    } else {
                        break;
                    }
                }
                if self.peek() == Some('*') && matches!(name.as_str(), "begin" | "end") {}
            }
            Some(c) => {
                name.push(c);
                self.i += 1;
            }
            None => {}
        }
        name
    }

    fn parse_row(&mut self, stop: Stop) -> Node {
        self.stops.push(stop);
        self.depth += 1;
        let mut items: Vec<Node> = Vec::new();
        if self.depth > 60 {
            self.errors.push("lồng quá sâu".into());
            self.i = self.s.len();
        }
        loop {
            self.skip_ws();
            if self.peek().is_none() || self.at_stop() {
                break;
            }
            if stop == Stop::Top && self.peek() == Some('}') {
                self.i += 1; // dau } thua
                continue;
            }
            if self.starts("\\\\") {
                self.i += 2;
                // bo qua tham so [2pt]
                if self.peek() == Some('[') {
                    while let Some(c) = self.peek() {
                        self.i += 1;
                        if c == ']' {
                            break;
                        }
                    }
                }
                items.push(Node::Newline);
                continue;
            }
            let atom = self.parse_atom();
            let Some(atom) = atom else { continue };
            let node = self.postfix(atom);
            items.push(node);
        }
        self.depth -= 1;
        self.stops.pop();
        if items.len() == 1 {
            items.pop().unwrap()
        } else {
            Node::Row(items)
        }
    }

    fn postfix(&mut self, base: Node) -> Node {
        let mut sup: Option<Box<Node>> = None;
        let mut sub: Option<Box<Node>> = None;
        loop {
            self.skip_ws();
            match self.peek() {
                Some('^') => {
                    self.i += 1;
                    let a = self.script_arg();
                    sup = Some(Box::new(match sup.take() {
                        Some(old) => Node::Row(vec![*old, a]),
                        None => a,
                    }));
                }
                Some('_') => {
                    self.i += 1;
                    let a = self.script_arg();
                    sub = Some(Box::new(a));
                }
                Some('\'') | Some('’') => {
                    self.i += 1;
                    let prime = Node::Sym("′".into());
                    sup = Some(Box::new(match sup.take() {
                        Some(old) => Node::Row(vec![*old, prime]),
                        None => prime,
                    }));
                }
                _ => break,
            }
        }
        if sup.is_none() && sub.is_none() {
            base
        } else {
            Node::Script { base: Box::new(base), sup, sub }
        }
    }

    fn script_arg(&mut self) -> Node {
        self.skip_ws();
        match self.peek() {
            Some('{') => {
                self.i += 1;
                let n = self.parse_row(Stop::Brace);
                if self.peek() == Some('}') {
                    self.i += 1;
                }
                n
            }
            // TeX: doi so khong co ngoac chi la MOT ky tu (\frac13 = 1/3, x^23 = x^2 3)
            Some(c) if c.is_ascii_digit() => {
                self.i += 1;
                Node::Num(c.to_string())
            }
            Some(_) => self.parse_atom().unwrap_or(Node::Row(vec![])),
            None => Node::Row(vec![]),
        }
    }

    /// doi so cua lenh: {…} hoac mot ky tu/lenh
    fn arg(&mut self) -> Node {
        self.script_arg()
    }

    fn raw_group(&mut self) -> String {
        // doc noi dung tho trong {...} (cho \text, \begin{env})
        self.skip_ws();
        let mut out = String::new();
        if self.peek() == Some('{') {
            self.i += 1;
            let mut depth = 1;
            while let Some(c) = self.peek() {
                self.i += 1;
                if c == '{' {
                    depth += 1;
                } else if c == '}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                out.push(c);
            }
        } else if let Some(c) = self.peek() {
            self.i += 1;
            out.push(c);
        }
        out
    }

    fn read_delim(&mut self) -> String {
        self.skip_ws();
        match self.peek() {
            Some('\\') => {
                self.i += 1;
                let n = self.read_name();
                match n.as_str() {
                    "{" | "lbrace" => "{".into(),
                    "}" | "rbrace" => "}".into(),
                    "|" | "Vert" | "lVert" | "rVert" => "‖".into(),
                    "langle" => "⟨".into(),
                    "rangle" => "⟩".into(),
                    "lfloor" => "⌊".into(),
                    "rfloor" => "⌋".into(),
                    "lceil" => "⌈".into(),
                    "rceil" => "⌉".into(),
                    "vert" | "lvert" | "rvert" => "|".into(),
                    _ => String::new(),
                }
            }
            Some('.') => {
                self.i += 1;
                String::new()
            }
            Some(c) => {
                self.i += 1;
                c.to_string()
            }
            None => String::new(),
        }
    }

    fn parse_atom(&mut self) -> Option<Node> {
        let c = self.peek()?;
        self.i += 1;
        Some(match c {
            '{' => {
                let n = self.parse_row(Stop::Brace);
                if self.peek() == Some('}') {
                    self.i += 1;
                }
                n
            }
            '\\' => return self.command(),
            '^' | '_' => Node::Sym(c.to_string()),
            '~' => Node::Space(0.28),
            c if c.is_ascii_digit() => {
                let mut n = c.to_string();
                while let Some(d) = self.peek() {
                    if d.is_ascii_digit() {
                        n.push(d);
                        self.i += 1;
                    } else if (d == '.' || d == ',') && matches!(self.s.get(self.i + 1), Some(x) if x.is_ascii_digit()) {
                        n.push(d);
                        self.i += 1;
                    } else {
                        break;
                    }
                }
                Node::Num(n)
            }
            '(' | ')' | '[' | ']' | ',' | ';' | ':' | '!' | '?' | '.' | '|' | '/' => {
                if c == '/' {
                    Node::Op("/".into(), false)
                } else {
                    Node::Sym(c.to_string())
                }
            }
            other => classify_char(other),
        })
    }

    fn command(&mut self) -> Option<Node> {
        let name = self.read_name();
        if name.is_empty() {
            return None;
        }
        // khoang cach
        match name.as_str() {
            "," => return Some(Node::Space(0.17)),
            ":" | ">" => return Some(Node::Space(0.22)),
            ";" => return Some(Node::Space(0.28)),
            "!" => return Some(Node::Space(-0.17)),
            " " => return Some(Node::Space(0.28)),
            "quad" => return Some(Node::Space(1.0)),
            "qquad" => return Some(Node::Space(2.0)),
            "displaystyle" | "textstyle" | "scriptstyle" | "limits" | "nolimits" | "nonumber" | "notag" | "big" | "Big"
            | "bigg" | "Bigg" | "bigl" | "bigr" | "Bigl" | "Bigr" | "biggl" | "biggr" | "protect" | "mathop" | "mathrel"
            | "mathbin" | "mathord" | "hline" | "centering" => return None,
            "label" | "tag" | "color" | "textcolor" | "colorbox" | "pagecolor" => {
                if name == "textcolor" || name == "colorbox" {
                    let _ = self.raw_group();
                    return Some(self.arg());
                }
                let _ = self.raw_group();
                return None;
            }
            _ => {}
        }
        if let Some((u, k)) = lookup_sym(&name) {
            return Some(match k {
                K::Id => Node::Ident(u.into()),
                K::Sym => Node::Sym(u.into()),
                K::Bin => Node::Op(u.into(), false),
                K::Rel => Node::Op(u.into(), true),
                K::Big => Node::BigOp(u.into()),
            });
        }
        if FUNCS.contains(&name.as_str()) {
            return Some(Node::Func(name));
        }
        match name.as_str() {
            "frac" | "dfrac" | "tfrac" | "cfrac" => {
                let a = self.arg();
                let b = self.arg();
                Some(Node::Frac(Box::new(a), Box::new(b), true))
            }
            "binom" | "dbinom" | "tbinom" => {
                let a = self.arg();
                let b = self.arg();
                Some(Node::Matrix { rows: vec![vec![a], vec![b]], open: "(".into(), close: ")".into(), align: vec!['c'] })
            }
            "sqrt" => {
                self.skip_ws();
                let mut idx = None;
                if self.peek() == Some('[') {
                    self.i += 1;
                    let mut raw = String::new();
                    while let Some(c) = self.peek() {
                        self.i += 1;
                        if c == ']' {
                            break;
                        }
                        raw.push(c);
                    }
                    idx = Some(Box::new(parse_inner(&raw)));
                }
                let b = self.arg();
                Some(Node::Sqrt(idx, Box::new(b)))
            }
            "left" => {
                let l = self.read_delim();
                let inner = self.parse_row(Stop::Right);
                let mut r = String::new();
                if self.starts("\\right") {
                    self.i += 6;
                    r = self.read_delim();
                }
                Some(Node::Delim(l, Box::new(inner), r))
            }
            "right" => None, // thua
            "text" | "textrm" | "mbox" | "textnormal" | "textit" | "textbf" | "hbox" | "mathrm" | "operatorname" | "mathsf" | "mathtt" => {
                let raw = self.raw_group();
                if matches!(name.as_str(), "mathrm" | "operatorname") {
                    // trong \mathrm co the co lenh con -> phan tich lai, ep thang
                    let inner = parse_inner(&raw);
                    let n = Node::Styled(Sty { bold: false, roman: true }, Box::new(inner));
                    return Some(if name == "operatorname" { Node::Func(flatten_text(&n)) } else { n });
                }
                Some(Node::Text(raw))
            }
            "mathbf" | "boldsymbol" | "bm" | "mathbfit" => {
                let a = self.arg();
                Some(Node::Styled(Sty { bold: true, roman: name == "mathbf" }, Box::new(a)))
            }
            "mathit" | "mathnormal" => {
                let a = self.arg();
                Some(a)
            }
            "mathbb" | "Bbb" => {
                let raw = self.raw_group();
                let mut out = Vec::new();
                for ch in raw.chars() {
                    match blackboard(ch) {
                        Some(b) => out.push(Node::Sym(b.into())),
                        None => out.push(Node::Styled(Sty { bold: false, roman: true }, Box::new(classify_char(ch)))),
                    }
                }
                Some(if out.len() == 1 { out.pop().unwrap() } else { Node::Row(out) })
            }
            "mathcal" | "mathscr" | "mathfrak" => {
                let a = self.arg();
                Some(a)
            }
            "hat" | "widehat" => Some(Node::Accent(Accent::Hat, Box::new(self.arg()))),
            "bar" | "overline" => Some(Node::Accent(Accent::Bar, Box::new(self.arg()))),
            "vec" | "overrightarrow" | "overrightharpoon" => Some(Node::Accent(Accent::Vec, Box::new(self.arg()))),
            "dot" => Some(Node::Accent(Accent::Dot, Box::new(self.arg()))),
            "ddot" => Some(Node::Accent(Accent::DDot, Box::new(self.arg()))),
            "tilde" | "widetilde" => Some(Node::Accent(Accent::Tilde, Box::new(self.arg()))),
            "underline" => Some(Node::Accent(Accent::Underline, Box::new(self.arg()))),
            "overbrace" => Some(Node::Accent(Accent::OverBrace, Box::new(self.arg()))),
            "underbrace" => Some(Node::Accent(Accent::UnderBrace, Box::new(self.arg()))),
            "overset" | "stackrel" => {
                let top = self.arg();
                let base = self.arg();
                Some(Node::Over(Box::new(base), Box::new(top)))
            }
            "underset" => {
                let bot = self.arg();
                let base = self.arg();
                Some(Node::Under(Box::new(base), Box::new(bot)))
            }
            "xrightarrow" | "xleftarrow" => {
                self.skip_ws();
                let mut under = None;
                if self.peek() == Some('[') {
                    self.i += 1;
                    let mut raw = String::new();
                    while let Some(c) = self.peek() {
                        self.i += 1;
                        if c == ']' {
                            break;
                        }
                        raw.push(c);
                    }
                    under = Some(parse_inner(&raw));
                }
                let over = self.arg();
                let arrow = Node::Op(if name == "xrightarrow" { "→".into() } else { "←".into() }, true);
                let mut n = Node::Over(Box::new(arrow), Box::new(over));
                if let Some(u) = under {
                    n = Node::Under(Box::new(n), Box::new(u));
                }
                Some(n)
            }
            "substack" => {
                let raw = self.raw_group();
                Some(parse_inner(&raw.replace("\\\\", "\\\\")))
            }
            "begin" => {
                let env = self.raw_group();
                Some(self.environment(&env))
            }
            "end" => {
                let _ = self.raw_group();
                None
            }
            "\\" => Some(Node::Newline),
            _ => {
                self.errors.push(format!("\\{name}"));
                Some(Node::Styled(Sty { bold: false, roman: true }, Box::new(Node::Text(format!("\\{name}")))))
            }
        }
    }

    fn environment(&mut self, env: &str) -> Node {
        let env_base = env.trim_end_matches('*');
        let (open, close, align): (&str, &str, Vec<char>) = match env_base {
            "pmatrix" => ("(", ")", vec!['c']),
            "bmatrix" => ("[", "]", vec!['c']),
            "Bmatrix" => ("{", "}", vec!['c']),
            "vmatrix" => ("|", "|", vec!['c']),
            "Vmatrix" => ("‖", "‖", vec!['c']),
            "matrix" | "smallmatrix" | "array" | "subarray" => ("", "", vec!['c']),
            "cases" | "dcases" | "rcases" => ("{", "", vec!['l']),
            "aligned" | "align" | "alignat" | "split" | "eqnarray" | "flalign" => ("", "", vec!['r', 'l']),
            _ => ("", "", vec!['c']), // equation, gather, multline...
        };
        // array{cc}: bo qua tham so cot
        if env_base == "array" || env_base == "subarray" {
            let _ = self.raw_group();
        }
        let mut rows: Vec<Vec<Node>> = Vec::new();
        let mut cur: Vec<Node> = Vec::new();
        loop {
            let cell = self.parse_row(Stop::Cell);
            cur.push(cell);
            if self.peek() == Some('&') {
                self.i += 1;
                continue;
            }
            if self.starts("\\\\") {
                self.i += 2;
                if self.peek() == Some('[') {
                    while let Some(c) = self.peek() {
                        self.i += 1;
                        if c == ']' {
                            break;
                        }
                    }
                }
                rows.push(std::mem::take(&mut cur));
                continue;
            }
            if self.starts("\\end") {
                self.i += 4;
                let _ = self.raw_group();
                if !cur.is_empty() {
                    rows.push(std::mem::take(&mut cur));
                }
                break;
            }
            // het chuoi
            if !cur.is_empty() {
                rows.push(std::mem::take(&mut cur));
            }
            break;
        }
        // bo hang trong cuoi
        while rows.last().map_or(false, |r| r.iter().all(|c| matches!(c, Node::Row(v) if v.is_empty()))) && rows.len() > 1 {
            rows.pop();
        }
        if matches!(env_base, "equation" | "gather" | "multline" | "displaymath" | "math") && rows.len() == 1 && rows[0].len() == 1 {
            return rows.pop().unwrap().pop().unwrap();
        }
        Node::Matrix { rows, open: open.into(), close: close.into(), align }
    }
}

fn parse_inner(src: &str) -> Node {
    let mut p = P { s: src.chars().collect(), i: 0, errors: vec![], stops: vec![], depth: 0 };
    p.parse_row(Stop::Top)
}

fn flatten_text(n: &Node) -> String {
    match n {
        Node::Ident(s) | Node::Num(s) | Node::Sym(s) | Node::Text(s) | Node::Func(s) | Node::BigOp(s) => s.clone(),
        Node::Op(s, _) => s.clone(),
        Node::Row(v) => v.iter().map(flatten_text).collect(),
        Node::Styled(_, b) => flatten_text(b),
        _ => String::new(),
    }
}

pub fn parse(src: &str) -> Parsed {
    let mut p = P { s: src.chars().collect(), i: 0, errors: vec![], stops: vec![], depth: 0 };
    let root = p.parse_row(Stop::Top);
    Parsed { root, errors: p.errors }
}

// =================================================================== bo cuc ve
enum DrawOp {
    Text(Pos2, Arc<egui::Galley>),
    Poly(Vec<Pos2>, f32),
    Dot(Pos2, f32),
}

pub struct MathBox {
    pub w: f32,
    pub asc: f32,
    pub desc: f32,
    pub has_error: bool,
    ops: Vec<DrawOp>,
    color: Color32,
}

#[derive(Default)]
struct MBox {
    w: f32,
    asc: f32,
    desc: f32,
    ops: Vec<DrawOp>,
}

impl MBox {
    fn h(&self) -> f32 {
        self.asc + self.desc
    }
    fn push_child(&mut self, c: MBox, dx: f32, dy: f32) {
        for op in c.ops {
            self.ops.push(match op {
                DrawOp::Text(p, g) => DrawOp::Text(p + Vec2::new(dx, dy), g),
                DrawOp::Poly(pts, w) => DrawOp::Poly(pts.into_iter().map(|p| p + Vec2::new(dx, dy)).collect(), w),
                DrawOp::Dot(p, r) => DrawOp::Dot(p + Vec2::new(dx, dy), r),
            });
        }
    }
}

impl MathBox {
    pub fn paint(&self, painter: &egui::Painter, origin: Pos2) {
        let stroke_color = self.color;
        for op in &self.ops {
            match op {
                DrawOp::Text(p, g) => {
                    painter.galley(origin + p.to_vec2(), g.clone(), stroke_color);
                }
                DrawOp::Poly(pts, w) => {
                    let pts: Vec<Pos2> = pts.iter().map(|p| origin + p.to_vec2()).collect();
                    painter.add(Shape::line(pts, Stroke::new(*w, stroke_color)));
                }
                DrawOp::Dot(p, r) => {
                    painter.circle_filled(origin + p.to_vec2(), *r, stroke_color);
                }
            }
        }
    }
}

struct L<'a> {
    ctx: &'a egui::Context,
    color: Color32,
    err: std::cell::Cell<bool>,
}

#[derive(Clone, Copy, PartialEq)]
enum Prev {
    Start,
    Bin,
    Rel,
    Open,
    Ord,
}

fn lw(sz: f32) -> f32 {
    (sz * 0.07).max(1.0)
}

impl<'a> L<'a> {
    fn text(&self, s: &str, sz: f32, italic: bool, bold: bool, color: Option<Color32>) -> MBox {
        if s.is_empty() {
            return MBox::default();
        }
        let family = if bold { FontFamily::Name("InterSemiBold".into()) } else { FontFamily::Proportional };
        let mut job = LayoutJob::default();
        job.append(
            s,
            0.0,
            TextFormat {
                font_id: FontId::new(sz, family),
                color: color.unwrap_or(self.color),
                italics: italic,
                ..Default::default()
            },
        );
        job.wrap.max_width = f32::INFINITY;
        let galley = self.ctx.fonts(|f| f.layout_job(job));
        let row_h = galley.size().y;
        let asc = 0.76 * sz;
        let desc = 0.22 * sz;
        let base_off = 0.80 * row_h;
        MBox { w: galley.size().x, asc, desc, ops: vec![DrawOp::Text(Pos2::new(0.0, asc - base_off), galley)] }
    }

    fn sp(&self, w: f32) -> MBox {
        MBox { w, asc: 0.0, desc: 0.0, ops: vec![] }
    }

    fn lay(&self, n: &Node, sz: f32, st: Sty, disp: bool) -> MBox {
        match n {
            Node::Ident(s) => self.text(s, sz, !st.roman, st.bold, None),
            Node::Num(s) | Node::Sym(s) => self.text(s, sz, false, st.bold, None),
            Node::Op(s, _) => self.text(s, sz, false, st.bold, None),
            Node::Text(s) => self.text(s, sz, false, st.bold, None),
            Node::Func(s) => {
                let mut b = self.text(s, sz, false, false, None);
                b.w += 0.1 * sz;
                b
            }
            Node::BigOp(s) => {
                let big = if disp { sz * 1.45 } else { sz * 1.15 };
                let mut b = self.text(s, big, false, false, None);
                // can giua theo truc toan hoc
                let shift = if disp { 0.20 * sz } else { 0.06 * sz };
                b.asc = (b.asc - shift).max(0.5 * sz);
                b.desc = b.desc + shift * 0.6;
                for op in b.ops.iter_mut() {
                    if let DrawOp::Text(p, _) = op {
                        p.y -= 0.0;
                    }
                }
                b.w += 0.08 * sz;
                b
            }
            Node::Space(w) => {
                if *w < 0.0 {
                    MBox { w: 0.0, ..Default::default() }
                } else {
                    self.sp(w * sz)
                }
            }
            Node::Newline => MBox::default(),
            Node::Styled(s2, inner) => {
                let ns = Sty { bold: st.bold || s2.bold, roman: st.roman || s2.roman };
                self.lay(inner, sz, ns, disp)
            }
            Node::Row(items) => self.lay_row(items, sz, st, disp),
            Node::Script { base, sup, sub } => self.lay_script(base, sup.as_deref(), sub.as_deref(), sz, st, disp),
            Node::Frac(a, b, line) => {
                let fs = sz * 0.88;
                let num = self.lay(a, fs, st, false);
                let den = self.lay(b, fs, st, false);
                let axis = 0.26 * sz;
                let gap = 0.16 * sz;
                let w = num.w.max(den.w) + 0.24 * sz;
                let asc = axis + gap + num.h();
                let desc = den.h() + gap - axis;
                let mut out = MBox { w, asc: asc.max(0.76 * sz), desc: desc.max(0.22 * sz), ops: vec![] };
                let bar_y = out.asc - axis;
                let (nh, dh) = (num.h(), den.h());
                let nx = (w - num.w) / 2.0;
                let dx = (w - den.w) / 2.0;
                out.push_child(num, nx, bar_y - gap - nh);
                out.push_child(den, dx, bar_y + gap);
                if *line {
                    out.ops.push(DrawOp::Poly(vec![Pos2::new(0.06 * sz, bar_y), Pos2::new(w - 0.06 * sz, bar_y)], lw(sz)));
                }
                let _ = (nh, dh);
                out
            }
            Node::Sqrt(idx, body) => {
                let b = self.lay(body, sz, st, disp);
                let rad_w = 0.62 * sz;
                let pad = 0.14 * sz;
                let asc = b.asc + pad + lw(sz);
                let desc = b.desc + 0.04 * sz;
                let h = asc + desc;
                let (iw, ib) = match idx {
                    Some(i) => {
                        let ib = self.lay(i, sz * 0.58, st, false);
                        (ib.w, Some(ib))
                    }
                    None => (0.0, None),
                };
                let x0 = (iw - 0.35 * rad_w).max(0.0);
                let mut out = MBox { w: x0 + rad_w + b.w + 0.1 * sz, asc, desc, ops: vec![] };
                let x_top = x0 + rad_w;
                let lwid = lw(sz) * 1.15;
                out.ops.push(DrawOp::Poly(
                    vec![
                        Pos2::new(x0, 0.58 * h),
                        Pos2::new(x0 + 0.2 * rad_w, 0.52 * h),
                        Pos2::new(x0 + 0.46 * rad_w, h - 0.02 * sz),
                        Pos2::new(x_top, lwid * 0.5),
                        Pos2::new(x_top + b.w + 0.1 * sz, lwid * 0.5),
                    ],
                    lwid,
                ));
                if let Some(ib) = ib {
                    let iy = (0.30 * h).min(h - ib.h());
                    out.push_child(ib, 0.0, iy.max(0.0));
                }
                let by = asc - b.asc;
                out.push_child(b, x_top + 0.05 * sz, by);
                out
            }
            Node::Delim(l, inner, r) => {
                let b = self.lay(inner, sz, st, disp);
                self.wrap_delims(l, r, b, sz)
            }
            Node::Matrix { rows, open, close, align } => {
                let compact = !open.is_empty() || !close.is_empty() || align.len() > 1;
                let col_gap = if align.len() > 1 { 0.3 * sz } else if compact { 0.9 * sz } else { 0.9 * sz };
                let row_gap = 0.32 * sz;
                let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
                let mut cells: Vec<Vec<MBox>> = Vec::new();
                for r in rows {
                    let mut line = Vec::new();
                    for c in r {
                        line.push(self.lay(c, sz, st, false));
                    }
                    cells.push(line);
                }
                let mut colw = vec![0.0f32; ncols];
                for line in &cells {
                    for (j, c) in line.iter().enumerate() {
                        colw[j] = colw[j].max(c.w);
                    }
                }
                let mut rh: Vec<(f32, f32)> = Vec::new();
                for line in &cells {
                    let a = line.iter().map(|c| c.asc).fold(0.76 * sz, f32::max);
                    let d = line.iter().map(|c| c.desc).fold(0.22 * sz, f32::max);
                    rh.push((a, d));
                }
                let total_h: f32 = rh.iter().map(|(a, d)| a + d).sum::<f32>() + row_gap * (rh.len().saturating_sub(1)) as f32;
                let total_w: f32 = colw.iter().sum::<f32>() + col_gap * (ncols.saturating_sub(1)) as f32;
                let axis = 0.26 * sz;
                let asc = total_h / 2.0 + axis;
                let desc = total_h - asc;
                let mut out = MBox { w: total_w, asc, desc, ops: vec![] };
                let mut y = 0.0;
                for (i, line) in cells.into_iter().enumerate() {
                    let (ra, rd) = rh[i];
                    let mut x = 0.0;
                    for (j, c) in line.into_iter().enumerate() {
                        let al = *align.get(j % align.len().max(1)).unwrap_or(&'c');
                        let off = match al {
                            'l' => 0.0,
                            'r' => colw[j] - c.w,
                            _ => (colw[j] - c.w) / 2.0,
                        };
                        let cy = y + ra - c.asc;
                        out.push_child(c, x + off, cy);
                        x += colw[j] + col_gap;
                    }
                    y += ra + rd + row_gap;
                }
                if open.is_empty() && close.is_empty() {
                    out
                } else {
                    self.wrap_delims(open, close, out, sz)
                }
            }
            Node::Accent(kind, inner) => {
                let b = self.lay(inner, sz, st, disp);
                self.lay_accent(kind, b, sz)
            }
            Node::Over(base, top) => {
                let b = self.lay(base, sz, st, disp);
                let t = self.lay(top, sz * 0.7, st, false);
                self.stack_vertical(Some(t), b, None, sz)
            }
            Node::Under(base, bot) => {
                let b = self.lay(base, sz, st, disp);
                let t = self.lay(bot, sz * 0.7, st, false);
                self.stack_vertical(None, b, Some(t), sz)
            }
        }
    }

    fn stack_vertical(&self, top: Option<MBox>, base: MBox, bot: Option<MBox>, sz: f32) -> MBox {
        let gap = 0.1 * sz;
        let w = base.w.max(top.as_ref().map_or(0.0, |t| t.w)).max(bot.as_ref().map_or(0.0, |t| t.w));
        let top_h = top.as_ref().map_or(0.0, |t| t.h() + gap);
        let bot_h = bot.as_ref().map_or(0.0, |t| t.h() + gap);
        let asc = base.asc + top_h;
        let desc = base.desc + bot_h;
        let mut out = MBox { w, asc, desc, ops: vec![] };
        let bw = base.w;
        let (basc, bdesc) = (base.asc, base.desc);
        out.push_child(base, (w - bw) / 2.0, top_h);
        if let Some(t) = top {
            let tw = t.w;
            out.push_child(t, (w - tw) / 2.0, 0.0);
        }
        if let Some(t) = bot {
            let tw = t.w;
            out.push_child(t, (w - tw) / 2.0, top_h + basc + bdesc + gap);
        }
        out
    }

    fn lay_accent(&self, kind: &Accent, b: MBox, sz: f32) -> MBox {
        let ext = 0.22 * sz;
        let w = b.w.max(0.3 * sz);
        let cx = w / 2.0;
        let lwid = lw(sz);
        match kind {
            Accent::Underline | Accent::UnderBrace => {
                let (basc, bdesc, bw) = (b.asc, b.desc, b.w);
                let mut out = MBox { w, asc: basc, desc: bdesc + ext, ops: vec![] };
                out.push_child(b, (w - bw) / 2.0, 0.0);
                let y = basc + bdesc + 0.1 * sz;
                if matches!(kind, Accent::UnderBrace) {
                    out.ops.push(DrawOp::Poly(
                        vec![
                            Pos2::new(0.0, y - 0.08 * sz),
                            Pos2::new(0.0, y),
                            Pos2::new(cx - 0.06 * sz, y),
                            Pos2::new(cx, y + 0.1 * sz),
                            Pos2::new(cx + 0.06 * sz, y),
                            Pos2::new(w, y),
                            Pos2::new(w, y - 0.08 * sz),
                        ],
                        lwid,
                    ));
                } else {
                    out.ops.push(DrawOp::Poly(vec![Pos2::new(0.0, y), Pos2::new(w, y)], lwid));
                }
                out
            }
            _ => {
                let (basc, bdesc, bw) = (b.asc, b.desc, b.w);
                let mut out = MBox { w, asc: basc + ext, desc: bdesc, ops: vec![] };
                out.push_child(b, (w - bw) / 2.0, ext);
                let y = 0.10 * sz;
                let hw = (w / 2.0).min(0.26 * sz);
                match kind {
                    Accent::Bar => out.ops.push(DrawOp::Poly(vec![Pos2::new(0.0, y + 0.04 * sz), Pos2::new(w, y + 0.04 * sz)], lwid)),
                    Accent::Hat => out.ops.push(DrawOp::Poly(
                        vec![Pos2::new(cx - hw, y + 0.1 * sz), Pos2::new(cx, y), Pos2::new(cx + hw, y + 0.1 * sz)],
                        lwid,
                    )),
                    Accent::Vec => {
                        out.ops.push(DrawOp::Poly(vec![Pos2::new(cx - w / 2.0 + 0.02 * sz, y + 0.06 * sz), Pos2::new(cx + w / 2.0, y + 0.06 * sz)], lwid));
                        out.ops.push(DrawOp::Poly(
                            vec![Pos2::new(cx + w / 2.0 - 0.1 * sz, y), Pos2::new(cx + w / 2.0, y + 0.06 * sz), Pos2::new(cx + w / 2.0 - 0.1 * sz, y + 0.12 * sz)],
                            lwid,
                        ));
                    }
                    Accent::Dot => out.ops.push(DrawOp::Dot(Pos2::new(cx, y + 0.05 * sz), lwid * 0.9)),
                    Accent::DDot => {
                        out.ops.push(DrawOp::Dot(Pos2::new(cx - 0.09 * sz, y + 0.05 * sz), lwid * 0.9));
                        out.ops.push(DrawOp::Dot(Pos2::new(cx + 0.09 * sz, y + 0.05 * sz), lwid * 0.9));
                    }
                    Accent::Tilde => {
                        let mut pts = Vec::new();
                        for i in 0..=10 {
                            let t = i as f32 / 10.0;
                            pts.push(Pos2::new(cx - hw + 2.0 * hw * t, y + 0.07 * sz - 0.05 * sz * (t * std::f32::consts::TAU).sin()));
                        }
                        out.ops.push(DrawOp::Poly(pts, lwid));
                    }
                    Accent::OverBrace => out.ops.push(DrawOp::Poly(
                        vec![
                            Pos2::new(0.0, y + 0.12 * sz),
                            Pos2::new(0.0, y + 0.04 * sz),
                            Pos2::new(cx - 0.06 * sz, y + 0.04 * sz),
                            Pos2::new(cx, y - 0.04 * sz),
                            Pos2::new(cx + 0.06 * sz, y + 0.04 * sz),
                            Pos2::new(w, y + 0.04 * sz),
                            Pos2::new(w, y + 0.12 * sz),
                        ],
                        lwid,
                    )),
                    _ => {}
                }
                out
            }
        }
    }

    fn wrap_delims(&self, l: &str, r: &str, b: MBox, sz: f32) -> MBox {
        let asc = b.asc.max(0.76 * sz);
        let desc = b.desc.max(0.22 * sz);
        let ext = 0.08 * sz;
        let (a2, d2) = (asc + ext, desc + ext);
        let left = self.delim_glyph(l, a2, d2, sz, false);
        let right = self.delim_glyph(r, a2, d2, sz, true);
        let w = left.w + b.w + right.w;
        let mut out = MBox { w, asc: a2.max(left.asc).max(right.asc), desc: d2.max(left.desc).max(right.desc), ops: vec![] };
        let (lw_, bw) = (left.w, b.w);
        let top = out.asc;
        let (ba, ld, rd_) = (b.asc, left.asc, right.asc);
        let _ = (ld, rd_);
        let lt = top - left.asc;
        let rt = top - right.asc;
        out.push_child(left, 0.0, lt);
        let bt = top - ba;
        out.push_child(b, lw_, bt);
        out.push_child(right, lw_ + bw, rt);
        out
    }

    fn delim_glyph(&self, ch: &str, asc: f32, desc: f32, sz: f32, right: bool) -> MBox {
        if ch.is_empty() {
            return MBox { w: 0.04 * sz, asc, desc, ops: vec![] };
        }
        let h = asc + desc;
        let normal = h <= 1.18 * sz && !matches!(ch, "⟨" | "⟩" | "⌊" | "⌋" | "⌈" | "⌉");
        if normal {
            let mut g = self.text(ch, sz, false, false, None);
            g.asc = asc;
            g.desc = desc;
            g.w += 0.04 * sz;
            // can lai theo duong co so
            for op in g.ops.iter_mut() {
                let _ = op;
            }
            return g;
        }
        let w = (0.18 * h).clamp(0.30 * sz, 0.55 * sz);
        let lwid = lw(sz);
        let m = |x: f32, y: f32| Pos2::new(x, y);
        let mirror = |x: f32| if right { w - x } else { x };
        let mut pts: Vec<Pos2> = Vec::new();
        let mut second: Option<Vec<Pos2>> = None;
        match ch {
            "(" | ")" => {
                let open_left = (ch == "(") ^ right;
                let _ = open_left;
                let steps = 16;
                for i in 0..=steps {
                    let t = i as f32 / steps as f32;
                    let u = 2.0 * t - 1.0;
                    let bulge = 1.0 - u * u;
                    let x = 0.85 * w - 0.7 * w * bulge.powf(0.8);
                    pts.push(m(if ch == "(" { x } else { w - x }, t * h));
                }
            }
            "[" | "]" => {
                let s = if ch == "[" { |x: f32| x } else { |x: f32| 1.0 - x };
                pts = vec![m(s(0.8) * w, 0.0), m(s(0.2) * w, 0.0), m(s(0.2) * w, h), m(s(0.8) * w, h)];
            }
            "{" | "}" => {
                let s = if ch == "{" { |x: f32| x } else { |x: f32| 1.0 - x };
                pts = vec![
                    m(s(0.85) * w, 0.0),
                    m(s(0.55) * w, 0.04 * h),
                    m(s(0.55) * w, 0.44 * h),
                    m(s(0.15) * w, 0.5 * h),
                    m(s(0.55) * w, 0.56 * h),
                    m(s(0.55) * w, 0.96 * h),
                    m(s(0.85) * w, h),
                ];
            }
            "|" => pts = vec![m(0.5 * w, 0.0), m(0.5 * w, h)],
            "‖" => {
                pts = vec![m(0.35 * w, 0.0), m(0.35 * w, h)];
                second = Some(vec![m(0.65 * w, 0.0), m(0.65 * w, h)]);
            }
            "⟨" | "⟩" => {
                let s = if ch == "⟨" { |x: f32| x } else { |x: f32| 1.0 - x };
                pts = vec![m(s(0.8) * w, 0.0), m(s(0.2) * w, 0.5 * h), m(s(0.8) * w, h)];
            }
            "⌊" => pts = vec![m(0.25 * w, 0.0), m(0.25 * w, h), m(0.8 * w, h)],
            "⌋" => pts = vec![m(0.2 * w, h), m(0.75 * w, h), m(0.75 * w, 0.0)],
            "⌈" => pts = vec![m(0.8 * w, 0.0), m(0.25 * w, 0.0), m(0.25 * w, h)],
            "⌉" => pts = vec![m(0.2 * w, 0.0), m(0.75 * w, 0.0), m(0.75 * w, h)],
            _ => {
                let mut g = self.text(ch, sz, false, false, None);
                g.asc = asc;
                g.desc = desc;
                return g;
            }
        }
        let _ = mirror;
        let mut ops = vec![DrawOp::Poly(pts, lwid)];
        if let Some(s) = second {
            ops.push(DrawOp::Poly(s, lwid));
        }
        MBox { w: w + 0.04 * sz, asc, desc, ops }
    }

    fn lay_script(&self, base: &Node, sup: Option<&Node>, sub: Option<&Node>, sz: f32, st: Sty, disp: bool) -> MBox {
        let limits = disp
            && match base {
                Node::BigOp(s) => is_limits_bigop(s),
                Node::Func(f) => is_limit_func(f),
                _ => false,
            };
        let b = self.lay(base, sz, st, disp);
        let ss = (sz * 0.70).max(6.5);
        let supb = sup.map(|n| self.lay(n, ss, st, false));
        let subb = sub.map(|n| self.lay(n, ss, st, false));
        if limits {
            return self.stack_vertical(supb, b, subb, sz);
        }
        let single = matches!(base, Node::Ident(_) | Node::Num(_) | Node::Sym(_) | Node::Op(..));
        let mut up = 0.0;
        let mut down = 0.0;
        if let Some(s) = &supb {
            up = if single { 0.40 * sz } else { (b.asc - 0.6 * s.asc).max(0.40 * sz) };
        }
        if let Some(s) = &subb {
            down = if single { 0.20 * sz } else { (b.desc * 0.9 + 0.06 * sz).max(0.20 * sz) };
            let _ = s;
        }
        if let (Some(a), Some(c)) = (&supb, &subb) {
            let gap = (down - c.asc) + (up - a.desc);
            if gap < 0.14 * sz {
                down += 0.14 * sz - gap;
            }
        }
        let sw = supb.as_ref().map_or(0.0, |s| s.w).max(subb.as_ref().map_or(0.0, |s| s.w));
        let asc = b.asc.max(supb.as_ref().map_or(0.0, |s| up + s.asc));
        let desc = b.desc.max(subb.as_ref().map_or(0.0, |s| down + s.desc));
        let bw = b.w;
        let mut out = MBox { w: bw + sw + 0.04 * sz, asc, desc, ops: vec![] };
        out.push_child(b, 0.0, asc - if true { out.asc } else { 0.0 } + 0.0);
        // b da duoc dat tai y = asc - b.asc ; tinh lai bang cach tinh truoc
        out
            .ops
            .clear();
        out
            .ops
            .shrink_to_fit();
        // (dat lai dung vi tri bang cach goi lai ham co cung kich thuoc)
        let b2 = self.lay(base, sz, st, disp);
        let b2_asc = b2.asc;
        out.push_child(b2, 0.0, asc - b2_asc);
        if let Some(s) = supb {
            let sa = s.asc;
            out.push_child(s, bw + 0.03 * sz, asc - up - sa);
        }
        if let Some(s) = subb {
            let sa = s.asc;
            out.push_child(s, bw + 0.01 * sz, asc + down - sa);
        }
        out
    }

    fn lay_row(&self, items: &[Node], sz: f32, st: Sty, disp: bool) -> MBox {
        // tach thanh cac dong theo Newline
        let mut lines: Vec<Vec<&Node>> = vec![vec![]];
        for n in items {
            if matches!(n, Node::Newline) {
                lines.push(vec![]);
            } else {
                lines.last_mut().unwrap().push(n);
            }
        }
        let mut boxes: Vec<MBox> = Vec::new();
        for line in lines {
            boxes.push(self.lay_line(&line, sz, st, disp));
        }
        if boxes.len() == 1 {
            return boxes.pop().unwrap();
        }
        let gap = 0.3 * sz;
        let w = boxes.iter().map(|b| b.w).fold(0.0, f32::max);
        let total: f32 = boxes.iter().map(|b| b.h()).sum::<f32>() + gap * (boxes.len() - 1) as f32;
        let asc = boxes[0].asc;
        let mut out = MBox { w, asc, desc: total - asc, ops: vec![] };
        let mut y = 0.0;
        for b in boxes {
            let h = b.h();
            out.push_child(b, 0.0, y);
            y += h + gap;
        }
        out
    }

    fn lay_line(&self, items: &[&Node], sz: f32, st: Sty, disp: bool) -> MBox {
        let mut parts: Vec<(MBox, f32)> = Vec::new(); // (box, x)
        let mut x = 0.0f32;
        let mut prev = Prev::Start;
        let mut asc = 0.0f32;
        let mut desc = 0.0f32;
        for n in items {
            let base_node: &Node = match n {
                Node::Script { base, .. } => base,
                other => other,
            };
            let (pre, post, kind) = match base_node {
                Node::Op(s, rel) => {
                    let unary = matches!(prev, Prev::Start | Prev::Bin | Prev::Rel | Prev::Open) && !*rel;
                    let _ = s;
                    if *rel {
                        (0.28 * sz, 0.28 * sz, Prev::Rel)
                    } else if unary {
                        (0.0, 0.0, Prev::Bin)
                    } else if s == "/" {
                        (0.0, 0.0, Prev::Bin)
                    } else {
                        (0.22 * sz, 0.22 * sz, Prev::Bin)
                    }
                }
                Node::Sym(s) if s == "," || s == ";" => (0.0, 0.17 * sz, Prev::Ord),
                Node::Sym(s) if s == "(" || s == "[" => (0.0, 0.0, Prev::Open),
                Node::Func(_) => (if prev == Prev::Ord { 0.12 * sz } else { 0.0 }, 0.0, Prev::Ord),
                Node::BigOp(_) => (if prev == Prev::Ord { 0.12 * sz } else { 0.0 }, 0.08 * sz, Prev::Ord),
                Node::Space(_) => (0.0, 0.0, prev),
                _ => (0.0, 0.0, Prev::Ord),
            };
            let b = self.lay(n, sz, st, disp);
            x += pre;
            asc = asc.max(b.asc);
            desc = desc.max(b.desc);
            let bw = b.w;
            parts.push((b, x));
            x += bw + post;
            prev = kind;
        }
        if parts.is_empty() {
            return MBox { w: 0.0, asc: 0.0, desc: 0.0, ops: vec![] };
        }
        let mut out = MBox { w: x, asc, desc, ops: vec![] };
        for (b, px) in parts {
            let top = asc - b.asc;
            out.push_child(b, px, top);
        }
        out
    }
}

// ================================================================= API cong khai
type Cache = HashMap<u64, Arc<MathBox>>;

/// Bo cuc cong thuc (co cache theo noi dung + co chu + mau + ppp).
pub fn layout(ctx: &egui::Context, src: &str, display: bool, size: f32, color: Color32) -> Arc<MathBox> {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    src.hash(&mut h);
    display.hash(&mut h);
    size.to_bits().hash(&mut h);
    color.to_array().hash(&mut h);
    ctx.pixels_per_point().to_bits().hash(&mut h);
    let key = h.finish();
    let id = egui::Id::new("mdt-math-cache");
    if let Some(hit) = ctx.data_mut(|d| d.get_temp_mut_or_default::<Cache>(id).get(&key).cloned()) {
        return hit;
    }
    let parsed = parse(src);
    let l = L { ctx, color, err: std::cell::Cell::new(false) };
    let b = l.lay(&parsed.root, size, Sty::default(), display);
    let mb = Arc::new(MathBox {
        w: b.w,
        asc: b.asc.max(0.5 * size),
        desc: b.desc.max(0.15 * size),
        has_error: !parsed.errors.is_empty() || l.err.get(),
        ops: b.ops,
        color,
    });
    ctx.data_mut(|d| {
        let c = d.get_temp_mut_or_default::<Cache>(id);
        if c.len() > 400 {
            c.clear();
        }
        c.insert(key, mb.clone());
    });
    mb
}

// ================================================================ OMML (Word)
fn xe(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            c if (c as u32) < 0x20 && c != '\t' && c != '\n' && c != '\r' => {}
            _ => o.push(c),
        }
    }
    o
}

fn m_run(text: &str, plain: bool, bold: bool) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut rpr = String::new();
    if plain || bold {
        rpr.push_str("<m:rPr>");
        if plain {
            rpr.push_str("<m:sty m:val=\"");
            rpr.push_str(if bold { "b" } else { "p" });
            rpr.push_str("\"/>");
        } else {
            rpr.push_str("<m:sty m:val=\"bi\"/>");
        }
        rpr.push_str("</m:rPr>");
    }
    format!(
        "<m:r>{rpr}<w:rPr><w:rFonts w:ascii=\"Cambria Math\" w:hAnsi=\"Cambria Math\"/></w:rPr><m:t xml:space=\"preserve\">{}</m:t></m:r>",
        xe(text)
    )
}

fn m_e(inner: &str) -> String {
    format!("<m:e>{inner}</m:e>")
}

fn omml_nodes(items: &[Node], st: Sty) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < items.len() {
        // toan tu lon: lay "toan hang" ke tiep lam than
        let (is_nary, big_sym, sub, sup) = match &items[i] {
            Node::BigOp(s) => (true, s.clone(), None, None),
            Node::Script { base, sup, sub } => match &**base {
                Node::BigOp(s) => (true, s.clone(), sub.as_deref(), sup.as_deref()),
                _ => (false, String::new(), None, None),
            },
            _ => (false, String::new(), None, None),
        };
        if is_nary {
            let mut j = i + 1;
            let mut body: Vec<Node> = Vec::new();
            while j < items.len() {
                match &items[j] {
                    Node::Op(_, _) | Node::Newline => break,
                    n => body.push(n.clone()),
                }
                j += 1;
            }
            let limloc = if is_limits_bigop(&big_sym) { "undOvr" } else { "subSup" };
            let sub_x = match sub {
                Some(n) => format!("<m:sub>{}</m:sub>", omml_node(n, st)),
                None => "<m:sub/>".into(),
            };
            let sup_x = match sup {
                Some(n) => format!("<m:sup>{}</m:sup>", omml_node(n, st)),
                None => "<m:sup/>".into(),
            };
            let hide = format!(
                "{}{}",
                if sub.is_none() { "<m:subHide m:val=\"1\"/>" } else { "" },
                if sup.is_none() { "<m:supHide m:val=\"1\"/>" } else { "" }
            );
            out.push_str(&format!(
                "<m:nary><m:naryPr><m:chr m:val=\"{}\"/><m:limLoc m:val=\"{limloc}\"/>{hide}</m:naryPr>{sub_x}{sup_x}{}</m:nary>",
                xe(&big_sym),
                m_e(&omml_nodes(&body, st))
            ));
            i = j;
            continue;
        }
        out.push_str(&omml_node(&items[i], st));
        i += 1;
    }
    out
}

fn omml_node(n: &Node, st: Sty) -> String {
    match n {
        Node::Ident(s) => m_run(s, st.roman, st.bold),
        Node::Num(s) | Node::Sym(s) | Node::Op(s, _) => m_run(s, true, st.bold),
        Node::Text(s) => m_run(s, true, st.bold),
        Node::Func(s) => m_run(s, true, false),
        Node::BigOp(s) => m_run(s, true, false),
        Node::Space(w) => {
            if *w <= 0.0 {
                String::new()
            } else {
                m_run(if *w >= 0.9 { "\u{2003}" } else { "\u{2009}" }, true, false)
            }
        }
        Node::Newline => String::new(),
        Node::Styled(s2, inner) => omml_node(inner, Sty { bold: st.bold || s2.bold, roman: st.roman || s2.roman }),
        Node::Row(items) => omml_nodes(items, st),
        Node::Script { base, sup, sub } => {
            // ham co gioi han (lim): dung m:limLow
            if let Node::Func(f) = &**base {
                if let (true, Some(s)) = (is_limit_func(f), sub.as_deref()) {
                    if sup.is_none() {
                        return format!(
                            "<m:limLow><m:limLowPr/><m:e>{}</m:e><m:lim>{}</m:lim></m:limLow>",
                            m_run(f, true, false),
                            omml_node(s, st)
                        );
                    }
                }
            }
            let b = omml_node(base, st);
            match (sup, sub) {
                (Some(p), Some(s)) => format!("<m:sSubSup><m:sSubSupPr/>{}<m:sub>{}</m:sub><m:sup>{}</m:sup></m:sSubSup>", m_e(&b), omml_node(s, st), omml_node(p, st)),
                (Some(p), None) => format!("<m:sSup><m:sSupPr/>{}<m:sup>{}</m:sup></m:sSup>", m_e(&b), omml_node(p, st)),
                (None, Some(s)) => format!("<m:sSub><m:sSubPr/>{}<m:sub>{}</m:sub></m:sSub>", m_e(&b), omml_node(s, st)),
                (None, None) => b,
            }
        }
        Node::Frac(a, b, line) => format!(
            "<m:f><m:fPr>{}</m:fPr><m:num>{}</m:num><m:den>{}</m:den></m:f>",
            if *line { "" } else { "<m:type m:val=\"noBar\"/>" },
            omml_node(a, st),
            omml_node(b, st)
        ),
        Node::Sqrt(idx, body) => match idx {
            Some(i) => format!("<m:rad><m:radPr/><m:deg>{}</m:deg>{}</m:rad>", omml_node(i, st), m_e(&omml_node(body, st))),
            None => format!("<m:rad><m:radPr><m:degHide m:val=\"1\"/></m:radPr><m:deg/>{}</m:rad>", m_e(&omml_node(body, st))),
        },
        Node::Delim(l, inner, r) => format!(
            "<m:d><m:dPr><m:begChr m:val=\"{}\"/><m:endChr m:val=\"{}\"/></m:dPr>{}</m:d>",
            xe(l),
            xe(r),
            m_e(&omml_node(inner, st))
        ),
        Node::Matrix { rows, open, close, align } => {
            let jc = match align.first() {
                Some('l') => "left",
                Some('r') => "right",
                _ => "center",
            };
            let mut s = format!("<m:m><m:mPr><m:mcs><m:mc><m:mcPr><m:mcJc m:val=\"{jc}\"/></m:mcPr></m:mc></m:mcs></m:mPr>");
            for r in rows {
                s.push_str("<m:mr>");
                for c in r {
                    s.push_str(&m_e(&omml_node(c, st)));
                }
                s.push_str("</m:mr>");
            }
            s.push_str("</m:m>");
            if open.is_empty() && close.is_empty() {
                s
            } else {
                format!(
                    "<m:d><m:dPr><m:begChr m:val=\"{}\"/><m:endChr m:val=\"{}\"/></m:dPr>{}</m:d>",
                    xe(open),
                    xe(close),
                    m_e(&s)
                )
            }
        }
        Node::Accent(k, inner) => {
            let e = omml_node(inner, st);
            match k {
                Accent::Bar => format!("<m:bar><m:barPr><m:pos m:val=\"top\"/></m:barPr>{}</m:bar>", m_e(&e)),
                Accent::Underline => format!("<m:bar><m:barPr><m:pos m:val=\"bot\"/></m:barPr>{}</m:bar>", m_e(&e)),
                Accent::OverBrace => format!("<m:groupChr><m:groupChrPr><m:chr m:val=\"⏞\"/><m:pos m:val=\"top\"/></m:groupChrPr>{}</m:groupChr>", m_e(&e)),
                Accent::UnderBrace => format!("<m:groupChr><m:groupChrPr><m:chr m:val=\"⏟\"/><m:pos m:val=\"bot\"/></m:groupChrPr>{}</m:groupChr>", m_e(&e)),
                other => {
                    let chr = match other {
                        Accent::Hat => "\u{0302}",
                        Accent::Vec => "\u{20D7}",
                        Accent::Dot => "\u{0307}",
                        Accent::DDot => "\u{0308}",
                        _ => "\u{0303}",
                    };
                    format!("<m:acc><m:accPr><m:chr m:val=\"{chr}\"/></m:accPr>{}</m:acc>", m_e(&e))
                }
            }
        }
        Node::Over(b, t) => format!(
            "<m:limUpp><m:limUppPr/>{}<m:lim>{}</m:lim></m:limUpp>",
            m_e(&omml_node(b, st)),
            omml_node(t, st)
        ),
        Node::Under(b, t) => format!(
            "<m:limLow><m:limLowPr/>{}<m:lim>{}</m:lim></m:limLow>",
            m_e(&omml_node(b, st)),
            omml_node(t, st)
        ),
    }
}

/// OMML cho mot cong thuc. `display` = true -> `<m:oMathPara>` (can giua, rieng mot dong).
pub fn to_omml(src: &str, display: bool) -> String {
    let p = parse(src);
    let inner = omml_node(&p.root, Sty::default());
    if display {
        format!("<m:oMathPara><m:oMath>{inner}</m:oMath></m:oMathPara>")
    } else {
        format!("<m:oMath>{inner}</m:oMath>")
    }
}

// =============================================================== MathML (HTML)
fn mml(n: &Node) -> String {
    match n {
        Node::Ident(s) => format!("<mi>{}</mi>", xe(s)),
        Node::Num(s) => format!("<mn>{}</mn>", xe(s)),
        Node::Sym(s) => format!("<mo>{}</mo>", xe(s)),
        Node::Op(s, _) => format!("<mo>{}</mo>", xe(s)),
        Node::Text(s) => format!("<mtext>{}</mtext>", xe(s)),
        Node::Func(s) => format!("<mi mathvariant=\"normal\">{}</mi>", xe(s)),
        Node::BigOp(s) => format!("<mo largeop=\"true\">{}</mo>", xe(s)),
        Node::Space(w) => format!("<mspace width=\"{:.2}em\"/>", w.max(0.0)),
        Node::Newline => "<mspace linebreak=\"newline\"/>".into(),
        Node::Styled(s, inner) => {
            let v = if s.bold { "bold" } else { "normal" };
            format!("<mstyle mathvariant=\"{v}\">{}</mstyle>", mml(inner))
        }
        Node::Row(v) => format!("<mrow>{}</mrow>", v.iter().map(mml).collect::<String>()),
        Node::Script { base, sup, sub } => {
            let b = mml(base);
            let limits = matches!(&**base, Node::BigOp(s) if is_limits_bigop(s)) || matches!(&**base, Node::Func(f) if is_limit_func(f));
            match (sup, sub) {
                (Some(p), Some(s)) => format!("<{t}>{b}{}{}</{t}>", mml(s), mml(p), t = if limits { "munderover" } else { "msubsup" }),
                (Some(p), None) => format!("<{t}>{b}{}</{t}>", mml(p), t = if limits { "mover" } else { "msup" }),
                (None, Some(s)) => format!("<{t}>{b}{}</{t}>", mml(s), t = if limits { "munder" } else { "msub" }),
                (None, None) => b,
            }
        }
        Node::Frac(a, b, line) => format!("<mfrac{}>{}{}</mfrac>", if *line { "" } else { " linethickness=\"0\"" }, mml(a), mml(b)),
        Node::Sqrt(i, b) => match i {
            Some(i) => format!("<mroot>{}{}</mroot>", mml(b), mml(i)),
            None => format!("<msqrt>{}</msqrt>", mml(b)),
        },
        Node::Delim(l, inner, r) => format!(
            "<mrow><mo stretchy=\"true\">{}</mo>{}<mo stretchy=\"true\">{}</mo></mrow>",
            xe(l),
            mml(inner),
            xe(r)
        ),
        Node::Matrix { rows, open, close, .. } => {
            let mut t = String::from("<mtable>");
            for r in rows {
                t.push_str("<mtr>");
                for c in r {
                    t.push_str(&format!("<mtd>{}</mtd>", mml(c)));
                }
                t.push_str("</mtr>");
            }
            t.push_str("</mtable>");
            if open.is_empty() && close.is_empty() {
                t
            } else {
                format!("<mrow><mo stretchy=\"true\">{}</mo>{t}<mo stretchy=\"true\">{}</mo></mrow>", xe(open), xe(close))
            }
        }
        Node::Accent(k, inner) => {
            let e = mml(inner);
            let (tag, chr) = match k {
                Accent::Hat => ("mover", "^"),
                Accent::Bar => ("mover", "¯"),
                Accent::Vec => ("mover", "→"),
                Accent::Dot => ("mover", "˙"),
                Accent::DDot => ("mover", "¨"),
                Accent::Tilde => ("mover", "~"),
                Accent::Underline => ("munder", "_"),
                Accent::OverBrace => ("mover", "⏞"),
                Accent::UnderBrace => ("munder", "⏟"),
            };
            format!("<{tag} accent=\"true\">{e}<mo>{chr}</mo></{tag}>")
        }
        Node::Over(b, t) => format!("<mover>{}{}</mover>", mml(b), mml(t)),
        Node::Under(b, t) => format!("<munder>{}{}</munder>", mml(b), mml(t)),
    }
}

pub fn to_mathml(src: &str, display: bool) -> String {
    let p = parse(src);
    format!(
        "<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"{}\">{}</math>",
        if display { "block" } else { "inline" },
        mml(&p.root)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic() {
        let p = parse(r"\frac{a+b}{c^2} + \sqrt[3]{x_i} = \sum_{i=1}^{n} i \cdot \alpha");
        assert!(p.errors.is_empty(), "{:?}", p.errors);
        let o = to_omml(r"\int_0^1 x^2\,dx", true);
        assert!(o.contains("<m:nary>"), "{o}");
        assert!(o.contains("m:oMathPara"));
        let m = to_mathml(r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}", false);
        assert!(m.contains("<mtable>"), "{m}");
        let u = parse(r"\foo + 1");
        assert_eq!(u.errors, vec!["\\foo".to_string()]);
    }

    #[test]
    fn single_digit_arg() {
        let m = to_mathml(r"\frac13", false);
        assert!(m.contains("<mn>1</mn><mn>3</mn></mfrac>"), "{m}");
    }

    #[test]
    fn omml_wellformed_chars() {
        let o = to_omml("a < b & c > d", false);
        assert!(!o.contains(" < "), "{o}");
    }
}
