use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::environment::Environment;
use crate::expr::{self, *};
use crate::lox_callable::{Callable, LoxCallable, LoxClass, LoxFunction};
use crate::report;
use crate::stmt::{self, *};
use crate::token::{LiteralTypes, Token, TokenType};

pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    pub environment: Rc<RefCell<Environment>>,
    locals: HashMap<Expr, usize>,
}

pub enum Exit {
    RuntimeError,
    Return(ReturnExit),
}

pub struct ReturnExit {
    pub value: LiteralTypes,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));
        Interpreter {
            globals: Rc::clone(&globals),
            environment: Rc::clone(&globals),
            locals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<(), Exit> {
        let mut has_error = false;
        for statement in statements.iter() {
            let s = self.execute(statement);
            match &s {
                Ok(_) => (),
                Err(e) => {
                    if let Exit::RuntimeError = e {
                        has_error = true;
                    }
                }
            }
        }

        if has_error {
            Err(Exit::RuntimeError {})
        } else {
            Ok(())
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), Exit> {
        stmt.accept(self)
    }

    pub fn resolve(&mut self, expr: &Expr, depth: usize) {
        self.locals.insert(expr.clone(), depth);
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<LiteralTypes, Exit> {
        expr.accept(self)
    }

    fn is_truthy(&self, ltype: &LiteralTypes) -> bool {
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
            LiteralTypes::Callable(c) => match c {
                Callable::Instance(ins) => ins.to_string(),
                Callable::Function(func) => func.to_string(),
                _ => "callable".to_string(),
            },
        }
    }

    pub fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Environment,
    ) -> Result<(), Exit> {
        let previous = Rc::clone(&self.environment);
        self.environment = Rc::new(RefCell::new(environment));

        let result = statements.iter().try_for_each(|stat| self.execute(stat));

        self.environment = previous;
        result
    }

    fn look_up_variable(&self, name: Token, expr: Expr) -> Result<LiteralTypes, Exit> {
        let distance = self.locals.get(&expr);
        if let Some(d) = distance {
            self.environment.borrow_mut().get_at(*d, name)
        } else {
            self.globals.borrow().get(&name)
        }
    }
}

impl stmt::Visitor<Result<(), Exit>> for Interpreter {
    fn visit_expression(&mut self, stmt: &Expression) -> Result<(), Exit> {
        self.evaluate(&stmt.expression)?;
        Ok(())
    }

    fn visit_print(&mut self, stmt: &Print) -> Result<(), Exit> {
        let value = self.evaluate(&stmt.expression)?;
        println!("{}", self.stringify(&value));
        Ok(())
    }

    fn visit_var(&mut self, stmt: &Var) -> Result<(), Exit> {
        let value = if let Expr::Literal(Literal {
            uuid: _usize,
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

    fn visit_block(&mut self, stmt: &Block) -> Result<(), Exit> {
        self.execute_block(
            &stmt.statements,
            Environment::new_with_enclosing(self.environment.clone()),
        )?;
        Ok(())
    }

    fn visit_if(&mut self, stmt: &If) -> Result<(), Exit> {
        let ltype = self.evaluate(&stmt.condition)?;
        if self.is_truthy(&ltype) {
            self.execute(&stmt.then_branch)?;
        } else if let Some(else_branch) = stmt.else_branch.as_ref() {
            self.execute(else_branch)?;
        }

        Ok(())
    }

    fn visit_while(&mut self, stmt: &While) -> Result<(), Exit> {
        loop {
            let ltype = self.evaluate(&stmt.condition)?;
            if !self.is_truthy(&ltype) {
                break;
            }
            self.execute(&stmt.body)?;
        }

        Ok(())
    }

    fn visit_function(&mut self, stmt: &Function) -> Result<(), Exit> {
        let function = LoxFunction::new(stmt.clone(), Rc::clone(&self.environment));
        self.environment.borrow_mut().define(
            stmt.name.lexeme.clone(),
            LiteralTypes::Callable(Callable::Function(function)),
        );
        Ok(())
    }

    fn visit_return(&mut self, stmt: &Return) -> Result<(), Exit> {
        let value = self.evaluate(&stmt.value)?;
        Err(Exit::Return(ReturnExit { value }))
    }

    fn visit_class(&mut self, stmt: &Class) -> Result<(), Exit> {
        self.environment
            .borrow_mut()
            .define(stmt.name.lexeme.clone(), LiteralTypes::Nil);

        let mut methods = HashMap::new();
        for method in stmt.methods.iter() {
            if let Stmt::Function(m) = method {
                let function = LoxFunction::new(m.clone(), Rc::clone(&self.environment));
                methods.insert(m.name.lexeme.clone(), function);
            }
        }

        let class = LoxClass::new(stmt.name.lexeme.clone(), methods);
        self.environment
            .borrow_mut()
            .assign(&stmt.name, LiteralTypes::Callable(Callable::Class(class)))?;
        Ok(())
    }
}

impl expr::Visitor<Result<LiteralTypes, Exit>> for Interpreter {
    fn visit_literal(&self, expr: &Literal) -> Result<LiteralTypes, Exit> {
        Ok(expr.value.clone())
    }

    fn visit_grouping(&mut self, expr: &Grouping) -> Result<LiteralTypes, Exit> {
        self.evaluate(&expr.expr)
    }

    fn visit_assignment(&mut self, expr: &Assignment) -> Result<LiteralTypes, Exit> {
        let value = self.evaluate(&expr.value)?;
        let distance = self.locals.get(&Expr::Assignment(expr.clone()));

        if let Some(d) = distance {
            self.environment
                .borrow_mut()
                .assign_at(*d, expr.name.clone(), value.clone());
        } else {
            self.globals
                .borrow_mut()
                .assign(&expr.name, value.clone())?;
        }
        Ok(value)
    }

    fn visit_unary(&mut self, expr: &Unary) -> Result<LiteralTypes, Exit> {
        let right = self.evaluate(&expr.right)?;

        match &expr.operator.ttype {
            TokenType::Minus => match right {
                LiteralTypes::Number(num) => Ok(LiteralTypes::Number(-num)),
                _ => {
                    report(expr.operator.line, "Operand must be a number.");
                    Err(Exit::RuntimeError {})
                }
            },
            TokenType::Bang => Ok(LiteralTypes::Bool(!self.is_truthy(&right))),
            _ => unreachable!(),
        }
    }

    fn visit_variable(&mut self, expr: &Variable) -> Result<LiteralTypes, Exit> {
        // self.environment.borrow().get(&expr.name)
        self.look_up_variable(expr.name.clone(), Expr::Variable(expr.clone()))
    }

    fn visit_call(&mut self, expr: &Call) -> Result<LiteralTypes, Exit> {
        let callee = self.evaluate(&expr.callee)?;
        let mut arguments = Vec::new();
        for argument in expr.arguments.iter() {
            arguments.push(self.evaluate(argument)?);
        }

        if let LiteralTypes::Callable(Callable::Function(function)) = callee {
            if arguments.len() != function.arity() {
                report(
                    expr.paren.line,
                    &format!(
                        "Expected {} arguments but got {}.",
                        function.arity(),
                        arguments.len()
                    ),
                );

                return Err(Exit::RuntimeError {});
            }

            function.call(self, &arguments)
        } else if let LiteralTypes::Callable(Callable::Class(class)) = callee {
            if arguments.len() != class.arity() {
                report(
                    expr.paren.line,
                    &format!(
                        "Expected {} arguments but got {}.",
                        class.arity(),
                        arguments.len()
                    ),
                );

                return Err(Exit::RuntimeError {});
            }

            class.call(self, &arguments)
        } else {
            report(expr.paren.line, "Can only call functions and classes.");
            Err(Exit::RuntimeError {})
        }
    }

    fn visit_get(&mut self, expr: &Get) -> Result<LiteralTypes, Exit> {
        let object = self.evaluate(&expr.object)?;

        if let LiteralTypes::Callable(Callable::Instance(mut ins)) = object {
            ins.get(&expr.name)
        } else {
            report(expr.name.line, "Only instances have properties.");
            Err(Exit::RuntimeError)
        }
    }

    fn visit_set(&mut self, expr: &Set) -> Result<LiteralTypes, Exit> {
        let object = self.evaluate(&expr.object)?;

        if let LiteralTypes::Callable(Callable::Instance(mut ins)) = object {
            let value = self.evaluate(&expr.value)?;
            ins.set(&expr.name, &value);
            Ok(value)
        } else {
            report(expr.name.line, "Only instances have fields.");
            Err(Exit::RuntimeError)
        }
    }

    fn visit_binary(&mut self, expr: &Binary) -> Result<LiteralTypes, Exit> {
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
                    Err(Exit::RuntimeError {})
                }
            }
            TokenType::Slash => {
                if let (LiteralTypes::Number(left_num), LiteralTypes::Number(right_num)) =
                    (left, right)
                {
                    Ok(LiteralTypes::Number(left_num / right_num))
                } else {
                    report(expr.operator.line, "Operands must be numbers.");
                    Err(Exit::RuntimeError {})
                }
            }
            TokenType::Star => {
                if let (LiteralTypes::Number(left_num), LiteralTypes::Number(right_num)) =
                    (left, right)
                {
                    Ok(LiteralTypes::Number(left_num * right_num))
                } else {
                    report(expr.operator.line, "Operands must be numbers.");
                    Err(Exit::RuntimeError {})
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
                    Err(Exit::RuntimeError {})
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
