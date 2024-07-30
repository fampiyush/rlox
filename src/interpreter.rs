use crate::expr::*;
use crate::report;
use crate::token::{LiteralTypes, TokenType};

pub struct Interpreter {}

pub struct RuntimeError {}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {}
    }

    pub fn interpret(&self, expr: &Expr) -> Result<LiteralTypes, RuntimeError> {
        self.evaluate(expr)
    }

    fn evaluate(&self, expr: &Expr) -> Result<LiteralTypes, RuntimeError> {
        expr.accept(self)
    }

    fn is_truthy(&self, ltype: LiteralTypes) -> bool {
        match &ltype {
            LiteralTypes::Nil => false,
            LiteralTypes::Bool(b) => *b,
            _ => true,
        }
    }

    fn is_equal(&self, left: &LiteralTypes, right: &LiteralTypes) -> bool {
        if *left == LiteralTypes::Nil && *right == LiteralTypes::Nil {
            return true;
        } else if *left == LiteralTypes::Nil {
            return false;
        }

        if let (LiteralTypes::Number(left_num), LiteralTypes::Number(right_num)) = (left, right) {
            left_num == right_num
        } else if let (LiteralTypes::String(left_str), LiteralTypes::String(right_str)) =
            (left, right)
        {
            left_str == right_str
        } else if let (LiteralTypes::Bool(left_bool), LiteralTypes::Bool(right_bool)) =
            (left, right)
        {
            left_bool == right_bool
        } else {
            false
        }
    }

    pub fn stringify(ltype: &LiteralTypes) -> String {
        match ltype {
            LiteralTypes::Nil => "nil".to_string(),
            LiteralTypes::Number(num) => {
                let mut text = num.to_string();
                if text.ends_with(".0") {
                    text = text[0..text.len() - 2].to_string();
                }
                text
            }
            LiteralTypes::String(s) => s.to_string(),
            LiteralTypes::Bool(b) => b.to_string(),
        }
    }
}

impl Visitor<Result<LiteralTypes, RuntimeError>> for Interpreter {
    fn visit_literal(&self, expr: &Literal) -> Result<LiteralTypes, RuntimeError> {
        Ok(expr.value.clone())
    }

    fn visit_grouping(&self, expr: &Grouping) -> Result<LiteralTypes, RuntimeError> {
        self.evaluate(&expr.expr)
    }

    fn visit_unary(&self, expr: &Unary) -> Result<LiteralTypes, RuntimeError> {
        let right = self.evaluate(&expr.right)?;

        match &expr.operator.ttype {
            TokenType::Minus => match right {
                LiteralTypes::Number(num) => Ok(LiteralTypes::Number(-num)),
                _ => {
                    report(expr.operator.line, "Operand must be a number.");
                    Err(RuntimeError {})
                }
            },
            TokenType::Bang => Ok(LiteralTypes::Bool(!self.is_truthy(right))),
            _ => unreachable!(),
        }
    }

    fn visit_binary(&self, expr: &Binary) -> Result<LiteralTypes, RuntimeError> {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;

        match &expr.operator.ttype {
            TokenType::Minus => {
                if let (LiteralTypes::Number(left_num), LiteralTypes::Number(right_num)) =
                    (left, right)
                {
                    Ok(LiteralTypes::Number(left_num - right_num))
                } else {
                    report(expr.operator.line, "Operands must be numbers.");
                    Err(RuntimeError {})
                }
            }
            TokenType::Slash => {
                if let (LiteralTypes::Number(left_num), LiteralTypes::Number(right_num)) =
                    (left, right)
                {
                    Ok(LiteralTypes::Number(left_num / right_num))
                } else {
                    report(expr.operator.line, "Operands must be numbers.");
                    Err(RuntimeError {})
                }
            }
            TokenType::Star => {
                if let (LiteralTypes::Number(left_num), LiteralTypes::Number(right_num)) =
                    (left, right)
                {
                    Ok(LiteralTypes::Number(left_num * right_num))
                } else {
                    report(expr.operator.line, "Operands must be numbers.");
                    Err(RuntimeError {})
                }
            }
            TokenType::Plus => match (left, right) {
                (LiteralTypes::Number(left_num), LiteralTypes::Number(right_num)) => {
                    Ok(LiteralTypes::Number(left_num + right_num))
                }
                (LiteralTypes::String(left_str), LiteralTypes::String(right_str)) => {
                    Ok(LiteralTypes::String(format!("{}{}", left_str, right_str)))
                }
                _ => {
                    report(
                        expr.operator.line,
                        "Operands must be two numbers or two strings.",
                    );
                    Err(RuntimeError {})
                }
            },
            TokenType::Greater => Ok(LiteralTypes::Bool(match (left, right) {
                (LiteralTypes::Number(left_num), LiteralTypes::Number(right_num)) => {
                    left_num > right_num
                }
                (LiteralTypes::String(left_str), LiteralTypes::String(right_str)) => {
                    left_str > right_str
                }
                _ => false,
            })),
            TokenType::GreaterEqual => Ok(LiteralTypes::Bool(match (left, right) {
                (LiteralTypes::Number(left_num), LiteralTypes::Number(right_num)) => {
                    left_num >= right_num
                }
                (LiteralTypes::String(left_str), LiteralTypes::String(right_str)) => {
                    left_str >= right_str
                }
                _ => false,
            })),
            TokenType::Less => Ok(LiteralTypes::Bool(match (left, right) {
                (LiteralTypes::Number(left_num), LiteralTypes::Number(right_num)) => {
                    left_num < right_num
                }
                (LiteralTypes::String(left_str), LiteralTypes::String(right_str)) => {
                    left_str < right_str
                }
                _ => false,
            })),
            TokenType::LessEqual => Ok(LiteralTypes::Bool(match (left, right) {
                (LiteralTypes::Number(left_num), LiteralTypes::Number(right_num)) => {
                    left_num <= right_num
                }
                (LiteralTypes::String(left_str), LiteralTypes::String(right_str)) => {
                    left_str <= right_str
                }
                _ => false,
            })),
            TokenType::BangEqual => Ok(LiteralTypes::Bool(!self.is_equal(&left, &right))),
            TokenType::EqualEqual => Ok(LiteralTypes::Bool(self.is_equal(&left, &right))),
            _ => unreachable!(),
        }
    }
}
