use serde::{Deserialize, Serialize};

/// Index of a basic block inside a [`NecroIR`] program.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BlockId(pub usize);

/// Byte and line offsets into the original source text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub byte_start: usize,
    pub byte_end: usize,
    pub line: usize,
}

impl Span {
    pub fn new(byte_start: usize, byte_end: usize, line: usize) -> Self {
        Span { byte_start, byte_end, line }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

/// Expressions are evaluated at the program's working scale (fixed point) for
/// COBOL and as 8-bit unsigned values for CHIP-8. The IR itself is untyped: the
/// origin-specific executors own the arithmetic rules.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Expr {
    Const(i64),
    Var(String),
    Bin(Box<Expr>, BinOp, Box<Expr>),
}

impl Expr {
    pub fn var(name: &str) -> Expr {
        Expr::Var(name.to_string())
    }
    pub fn bin(l: Expr, op: BinOp, r: Expr) -> Expr {
        Expr::Bin(Box::new(l), op, Box::new(r))
    }
    /// Collects every variable name read by the expression.
    pub fn reads(&self, out: &mut Vec<String>) {
        match self {
            Expr::Const(_) => {}
            Expr::Var(v) => out.push(v.clone()),
            Expr::Bin(l, _, r) => {
                l.reads(out);
                r.reads(out);
            }
        }
    }
    /// True when the expression is exactly `var + positive constant`.
    pub fn is_positive_increment_of(&self, var: &str) -> bool {
        match self {
            Expr::Bin(l, BinOp::Add, r) => {
                matches!((&**l, &**r), (Expr::Var(v), Expr::Const(c)) if v == var && *c > 0)
                    || matches!((&**l, &**r), (Expr::Const(c), Expr::Var(v)) if v == var && *c > 0)
            }
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoreMode {
    Truncate,
    Round,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Inst {
    /// `var := expr`, stored with the variable's own precision.
    Assign { var: String, expr: Expr, mode: StoreMode, span: Span },
    /// A buffer or table grows by one element (used for growth tracking).
    Append { var: String, span: Span },
    /// Side effect with no data-flow (DISPLAY, CLS, sound, ...).
    Effect { what: String, span: Span },
}

impl Inst {
    pub fn span(&self) -> Span {
        match self {
            Inst::Assign { span, .. } | Inst::Append { span, .. } | Inst::Effect { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Terminator {
    Jump(BlockId),
    Branch { cond: Expr, then_to: BlockId, else_to: BlockId },
    /// COBOL `PERFORM`: push `(end_para, ret)` on the perform stack and enter `entry`.
    Perform { entry: BlockId, end_para: usize, ret: BlockId },
    /// End of a COBOL paragraph: if the perform stack top names this paragraph,
    /// pop and return; otherwise fall through to `next`.
    ParaEnd { para: usize, next: BlockId },
    /// CHIP-8 `CALL`: push return address, jump.
    Call { target: BlockId, ret: BlockId },
    /// CHIP-8 `RET`: pop and jump to the pushed address.
    Return,
    Halt,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub id: BlockId,
    pub label: String,
    pub insts: Vec<Inst>,
    pub term: Terminator,
    pub span: Span,
}

/// Fixed-point storage description for a numeric variable.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VarDecl {
    pub name: String,
    /// Digits before the decimal point (COBOL PIC 9(n)); 3 for an 8-bit register.
    pub int_digits: u32,
    /// Digits after the decimal point (COBOL PIC V9(n)).
    pub scale: u32,
    pub signed: bool,
    /// True when the variable is an input the prover may seed.
    pub is_input: bool,
    pub initial: i64,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NecroIR {
    pub name: String,
    /// "cobol" or "chip8".
    pub origin: String,
    pub entry: BlockId,
    pub blocks: Vec<Block>,
    pub variables: Vec<VarDecl>,
    /// Working scale for fixed-point arithmetic (COBOL); 0 for CHIP-8.
    pub working_scale: u32,
    /// Human-readable names of COBOL paragraphs by index (empty for CHIP-8).
    pub paragraphs: Vec<String>,
}

impl NecroIR {
    pub fn block(&self, id: BlockId) -> &Block {
        &self.blocks[id.0]
    }
    pub fn var_index(&self, name: &str) -> Option<usize> {
        self.variables.iter().position(|v| v.name == name)
    }
    pub fn var(&self, name: &str) -> Option<&VarDecl> {
        self.variables.iter().find(|v| v.name == name)
    }
}
