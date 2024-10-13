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

/// Main expressions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Input(u64),
    Num(i64),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
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
            let op = match inner_rules.next().unwrap().as_str() {
                "+" => BinOp::Add,
                "-" => BinOp::Sub,
                "*" => BinOp::Mul,
                "/" => BinOp::Div,
                "==" => BinOp::Eq,
                _ => return Err("Unknown operator".to_string()),
            };
            let rhs = Box::new(convert_rule_to_expr(inner_rules.next().unwrap())?);
            Ok(Expr::BinOp(op, lhs, rhs))
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
