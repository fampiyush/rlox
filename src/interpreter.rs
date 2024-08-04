use std::cell::RefCell;
use std::rc::Rc;

use crate::environment::Environment;
use crate::expr::{self, *};
use crate::report;
use crate::stmt::{self, *};
use crate::token::{LiteralTypes, TokenType};

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

pub struct RuntimeError {}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        let mut has_error = false;
        for statement in statements.iter() {
            let s = self.execute(statement);
            match &s {
                Ok(_) => (),
                Err(_) => has_error = true,
            }
        }

        if has_error {
            Err(RuntimeError {})
        } else {
            Ok(())
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        stmt.accept(self)
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<LiteralTypes, RuntimeError> {
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

    pub fn stringify(&self, ltype: &LiteralTypes) -> String {
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

    fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Environment,
    ) -> Result<(), RuntimeError> {
        let previous = self.environment.clone();
        self.environment = Rc::new(RefCell::new(environment));
        let mut is_error = false;
        for statement in statements.iter() {
            let e = self.execute(statement);
            if e.is_err() {
                is_error = true
            }
        }
        self.environment = previous;
        if is_error {
            Err(RuntimeError {})
        } else {
            Ok(())
        }
    }
}

impl stmt::Visitor<Result<(), RuntimeError>> for Interpreter {
    fn visit_expression(&mut self, stmt: &Expression) -> Result<(), RuntimeError> {
        self.evaluate(&stmt.expression)?;
        Ok(())
    }

    fn visit_print(&mut self, stmt: &Print) -> Result<(), RuntimeError> {
        let value = self.evaluate(&stmt.expression)?;
        println!("{}", self.stringify(&value));
        Ok(())
    }

    fn visit_var(&mut self, stmt: &Var) -> Result<(), RuntimeError> {
        let value = if let Expr::Literal(Literal {
            value: LiteralTypes::Nil,
        }) = *stmt.initializer
        {
            LiteralTypes::Nil
        } else {
            self.evaluate(&stmt.initializer)?
        };
        self.environment
            .borrow_mut()
            .define(stmt.name.lexeme.clone(), value);
        Ok(())
    }

    fn visit_block(&mut self, stmt: &Block) -> Result<(), RuntimeError> {
        self.execute_block(
            &stmt.statements,
            Environment::new_with_enclosing(self.environment.clone()),
        )?;
        Ok(())
    }
}

impl expr::Visitor<Result<LiteralTypes, RuntimeError>> for Interpreter {
    fn visit_literal(&self, expr: &Literal) -> Result<LiteralTypes, RuntimeError> {
        Ok(expr.value.clone())
    }

    fn visit_grouping(&mut self, expr: &Grouping) -> Result<LiteralTypes, RuntimeError> {
        self.evaluate(&expr.expr)
    }

    fn visit_assignment(&mut self, expr: &Assignment) -> Result<LiteralTypes, RuntimeError> {
        let value = self.evaluate(&expr.value)?;
        self.environment
            .borrow_mut()
            .assign(&expr.name, value.clone())?;
        Ok(value)
    }

    fn visit_unary(&mut self, expr: &Unary) -> Result<LiteralTypes, RuntimeError> {
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

    fn visit_variable(&mut self, expr: &Variable) -> Result<LiteralTypes, RuntimeError> {
        self.environment.borrow().get(&expr.name)
    }

    fn visit_binary(&mut self, expr: &Binary) -> Result<LiteralTypes, RuntimeError> {
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
