#[derive(Debug)]
pub enum Expr {
    Literal(LiteralExpr),
    Identifier(String),
    BinaryOp(BinaryOpExpr),
    UnaryOp(UnaryOpExpr),
    FunctionCall(FunctionCallExpr),
    Lambda(LambdaExpr),
    Conditional(ConditionalExpr),
    Array(ArrayExpr),
    Object(ObjectExpr),
    // Add more expression types as needed
}

#[derive(Debug)]
pub enum LiteralValue {
    Int(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

#[derive(Debug)]
pub struct LiteralExpr {
    pub value: LiteralValue, // Can represent different types (int, float, string, bool)
}

impl LiteralExpr {
    pub fn new(value: LiteralValue) -> Self {
        Self { value }
    }
}

#[derive(Debug)]
pub struct BinaryOpExpr {
    pub left: Box<Expr>,
    pub operator: String, // e.g., "+", "-", "*", etc.
    pub right: Box<Expr>,
}

impl BinaryOpExpr {
    pub fn new(left: Expr, operator: String, right: Expr) -> Self {
        Self {
            left: left.into(),
            operator,
            right: right.into(),
        }
    }
}

#[derive(Debug)]
pub struct UnaryOpExpr {
    pub operator: String, // e.g., "-", "!", etc.
    pub operand: Box<Expr>,
}

impl UnaryOpExpr {
    pub fn new(operator: String, operand: Expr) -> Self {
        Self {
            operator,
            operand: operand.into(),
        }
    }
}

#[derive(Debug)]
pub struct FunctionCallExpr {
    pub function: Box<Expr>,  // Function name as an expression
    pub arguments: Vec<Expr>, // List of arguments
}

impl FunctionCallExpr {
    pub fn new(function: Expr, arguments: Vec<Expr>) -> Self {
        Self {
            function: function.into(),
            arguments,
        }
    }
}

#[derive(Debug)]
pub struct LambdaExpr {
    pub parameters: Vec<String>, // Names of parameters
    pub body: Box<Expr>,         // Expression for the body
}

impl LambdaExpr {
    pub fn new(parameters: Vec<String>, body: Expr) -> Self {
        Self {
            parameters,
            body: body.into(),
        }
    }
}

#[derive(Debug)]
pub struct ConditionalExpr {
    pub condition: Box<Expr>,   // The condition to evaluate
    pub then_branch: Box<Expr>, // Expression to evaluate if true
    pub else_branch: Box<Expr>, // Expression to evaluate if false
}

impl ConditionalExpr {
    pub fn new(condition: Expr, then_branch: Expr, else_branch: Expr) -> Self {
        Self {
            condition: condition.into(),
            then_branch: then_branch.into(),
            else_branch: else_branch.into(),
        }
    }
}

#[derive(Debug)]
pub struct ArrayExpr {
    pub elements: Vec<Expr>, // List of expressions
}

impl ArrayExpr {
    pub fn new(elements: Vec<Expr>) -> Self {
        Self { elements }
    }
}

#[derive(Debug)]
pub struct ObjectExpr {
    pub properties: Vec<(String, Expr)>, // Key-value pairs
}

impl ObjectExpr {
    pub fn new(properties: Vec<(String, Expr)>) -> Self {
        Self { properties }
    }
}
