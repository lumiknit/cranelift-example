use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "src/grammar.pest"]
struct Grammar;

/// Binary operator types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
}

impl BinOp {
    pub fn to_string(&self) -> &str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Eq => "==",
        }
    }

    pub fn from_string(op: &str) -> Option<Self> {
        match op {
            "+" => Some(BinOp::Add),
            "-" => Some(BinOp::Sub),
            "*" => Some(BinOp::Mul),
            "/" => Some(BinOp::Div),
            "==" => Some(BinOp::Eq),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltInFunc {
    Print,
    Rand,
}

impl BuiltInFunc {
    pub fn to_string(&self) -> &str {
        match self {
            BuiltInFunc::Print => "print",
            BuiltInFunc::Rand => "rand",
        }
    }

    pub fn from_string(name: &str) -> Option<Self> {
        match name {
            "print" => Some(BuiltInFunc::Print),
            "rand" => Some(BuiltInFunc::Rand),
            _ => None,
        }
    }
}

/// Main expressions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Input(u64),
    Num(i64),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    Call(BuiltInFunc, Box<Expr>),
}

fn convert_rule_to_expr(rule: pest::iterators::Pair<Rule>) -> Result<Expr, String> {
    match rule.as_rule() {
        Rule::root => convert_rule_to_expr(rule.into_inner().next().unwrap()),
        Rule::num => {
            let num_str = rule.as_str();
            let num = num_str
                .parse()
                .map_err(|e| format!("Failed to parse number: {}", e))?;
            Ok(Expr::Num(num))
        }
        Rule::calc => {
            let mut inner_rules = rule.into_inner();
            let lhs = Box::new(convert_rule_to_expr(inner_rules.next().unwrap())?);
            let op = BinOp::from_string(inner_rules.next().unwrap().as_str()).unwrap();
            let rhs = Box::new(convert_rule_to_expr(inner_rules.next().unwrap())?);
            Ok(Expr::BinOp(op, lhs, rhs))
        }
        Rule::call => {
            let mut inner_rules = rule.into_inner();
            let name = inner_rules.next().unwrap().as_str();
            let arg = convert_rule_to_expr(inner_rules.next().unwrap())?;
            let func = BuiltInFunc::from_string(name).unwrap();
            Ok(Expr::Call(func, Box::new(arg)))
        }
        Rule::input => {
            // Trim first '$' and parse index integer
            let index = rule.as_str()[1..]
                .parse()
                .map_err(|e| format!("Failed to parse index: {}", e))?;
            Ok(Expr::Input(index))
        }
        _ => Err(format!("Unknown rule: {:?}", rule)),
    }
}

/// Parse the input string into Expr
pub fn parse_expr(input: &str) -> Result<Expr, String> {
    // First, use pest to parse the input string
    let result = Grammar::parse(Rule::root, input).map_err(|e| e.to_string())?;

    // Then, convert the pest parse tree into Expr
    let expr = convert_rule_to_expr(result.into_iter().next().unwrap())?;

    // Finally, return the result
    Ok(expr)
}
