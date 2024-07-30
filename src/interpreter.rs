use crate::expr::*;
use crate::token::{LiteralTypes, TokenType};

struct Interpreter {}

impl Interpreter {
    fn evaluate(&self, expr: &Expr) -> LiteralTypes {
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
}

impl Visitor<LiteralTypes> for Interpreter {
    fn visit_literal(&self, expr: &Literal) -> LiteralTypes {
        expr.value.clone()
    }

    fn visit_grouping(&self, expr: &Grouping) -> LiteralTypes {
        self.evaluate(&expr.expr)
    }

    fn visit_unary(&self, expr: &Unary) -> LiteralTypes {
        let right = self.evaluate(&expr.right);

        match &expr.operator.ttype {
            TokenType::Minus => match right {
                LiteralTypes::Number(num) => LiteralTypes::Number(-num),
                _ => LiteralTypes::Nil,
            },
            TokenType::Bang => LiteralTypes::Bool(!self.is_truthy(right)),
            _ => LiteralTypes::Nil,
        }
    }

    fn visit_binary(&self, expr: &Binary) -> LiteralTypes {
        let left = self.evaluate(&expr.left);
        let right = self.evaluate(&expr.right);

        match &expr.operator.ttype {
            TokenType::Minus => {
                if let (LiteralTypes::Number(left_num), LiteralTypes::Number(right_num)) =
                    (left, right)
                {
                    LiteralTypes::Number(left_num - right_num)
                } else {
                    LiteralTypes::Nil
                }
            }
            TokenType::Slash => {
                if let (LiteralTypes::Number(left_num), LiteralTypes::Number(right_num)) =
                    (left, right)
                {
                    LiteralTypes::Number(left_num / right_num)
                } else {
                    LiteralTypes::Nil
                }
            }
            TokenType::Star => {
                if let (LiteralTypes::Number(left_num), LiteralTypes::Number(right_num)) =
                    (left, right)
                {
                    LiteralTypes::Number(left_num * right_num)
                } else {
                    LiteralTypes::Nil
                }
            }
            TokenType::Plus => match (left, right) {
                (LiteralTypes::Number(left_num), LiteralTypes::Number(right_num)) => {
                    LiteralTypes::Number(left_num + right_num)
                }
                (LiteralTypes::String(left_str), LiteralTypes::String(right_str)) => {
                    LiteralTypes::String(format!("{}{}", left_str, right_str))
                }
                _ => LiteralTypes::Nil,
            },
            TokenType::Greater => LiteralTypes::Bool(match (left, right) {
                (LiteralTypes::Number(left_num), LiteralTypes::Number(right_num)) => {
                    left_num > right_num
                }
                (LiteralTypes::String(left_str), LiteralTypes::String(right_str)) => {
                    left_str > right_str
                }
                _ => false,
            }),
            TokenType::GreaterEqual => LiteralTypes::Bool(match (left, right) {
                (LiteralTypes::Number(left_num), LiteralTypes::Number(right_num)) => {
                    left_num >= right_num
                }
                (LiteralTypes::String(left_str), LiteralTypes::String(right_str)) => {
                    left_str >= right_str
                }
                _ => false,
            }),
            TokenType::Less => LiteralTypes::Bool(match (left, right) {
                (LiteralTypes::Number(left_num), LiteralTypes::Number(right_num)) => {
                    left_num < right_num
                }
                (LiteralTypes::String(left_str), LiteralTypes::String(right_str)) => {
                    left_str < right_str
                }
                _ => false,
            }),
            TokenType::LessEqual => LiteralTypes::Bool(match (left, right) {
                (LiteralTypes::Number(left_num), LiteralTypes::Number(right_num)) => {
                    left_num <= right_num
                }
                (LiteralTypes::String(left_str), LiteralTypes::String(right_str)) => {
                    left_str <= right_str
                }
                _ => false,
            }),
            TokenType::BangEqual => LiteralTypes::Bool(!self.is_equal(&left, &right)),
            TokenType::EqualEqual => LiteralTypes::Bool(!self.is_equal(&left, &right)),
            _ => todo!(),
        }
    }
}
