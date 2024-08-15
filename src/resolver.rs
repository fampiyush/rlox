use std::collections::HashMap;

use crate::expr::Expr;
use crate::expr::*;
use crate::interpreter::Interpreter;
use crate::parser::ParserError;
use crate::stmt::*;
use crate::token::Token;

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

#[derive(Clone, Copy, PartialEq)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Clone, Copy, PartialEq)]
enum ClassType {
    None,
    Class,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Resolver {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    pub fn resolve_each(&mut self, statements: &[Stmt]) -> Result<(), ParserError> {
        for statement in statements.iter() {
            self.resolve_stmt(statement)?;
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, statement: &Stmt) -> Result<(), ParserError> {
        statement.accept(self)?;
        Ok(())
    }

    fn resolve_expr(&mut self, expression: &Expr) {
        let _ = expression.accept(self);
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: Token) -> Result<(), ParserError> {
        if !self.scopes.is_empty() {
            if self.scopes.last().unwrap().contains_key(&name.lexeme) {
                crate::error(name, "Already a variable with this name in this scope.");
                return Err(ParserError {});
            }
            self.scopes.last_mut().unwrap().insert(name.lexeme, false);
        }

        Ok(())
    }

    fn define(&mut self, name: Token) {
        if !self.scopes.is_empty() {
            self.scopes.last_mut().unwrap().insert(name.lexeme, true);
        }
    }

    fn resolve_local(&mut self, expr: &Expr, name: Token) {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(expr, self.scopes.len() - 1 - i);
            }
        }
    }

    fn resolve_function(
        &mut self,
        function: &Function,
        ftype: FunctionType,
    ) -> Result<(), ParserError> {
        let enclosing_fn = self.current_function;
        self.current_function = ftype;
        self.begin_scope();
        for param in function.params.iter() {
            self.declare(param.clone())?;
            self.define(param.clone());
        }
        self.resolve_each(&function.body)?;
        self.end_scope();
        self.current_function = enclosing_fn;
        Ok(())
    }
}

impl<'a> crate::stmt::Visitor<Result<(), ParserError>> for Resolver<'a> {
    fn visit_block(&mut self, stmt: &Block) -> Result<(), ParserError> {
        self.begin_scope();
        self.resolve_each(&stmt.statements)?;
        self.end_scope();

        Ok(())
    }

    fn visit_var(&mut self, stmt: &Var) -> Result<(), ParserError> {
        self.declare(stmt.name.clone())?;
        self.resolve_expr(stmt.initializer.as_ref());
        self.define(stmt.name.clone());

        Ok(())
    }

    fn visit_function(&mut self, stmt: &Function) -> Result<(), ParserError> {
        self.declare(stmt.name.clone())?;
        self.define(stmt.name.clone());

        self.resolve_function(stmt, FunctionType::Function)?;

        Ok(())
    }

    fn visit_expression(&mut self, stmt: &Expression) -> Result<(), ParserError> {
        self.resolve_expr(&stmt.expression);
        Ok(())
    }

    fn visit_if(&mut self, stmt: &If) -> Result<(), ParserError> {
        self.resolve_expr(&stmt.condition);
        self.resolve_stmt(&stmt.then_branch)?;
        if stmt.else_branch.is_some() {
            self.resolve_stmt(stmt.else_branch.as_ref().unwrap())?;
        }

        Ok(())
    }

    fn visit_print(&mut self, stmt: &Print) -> Result<(), ParserError> {
        self.resolve_expr(&stmt.expression);
        Ok(())
    }

    fn visit_return(&mut self, stmt: &Return) -> Result<(), ParserError> {
        if self.current_function == FunctionType::None {
            crate::error(stmt.keyword.clone(), "Can't return from top-level code.");
            return Err(ParserError {});
        } else if self.current_function == FunctionType::Initializer {
            dbg!(&stmt.value);
            crate::error(
                stmt.keyword.clone(),
                "Can't return a value from an initializer",
            );
            return Err(ParserError {});
        }

        self.resolve_expr(&stmt.value);
        Ok(())
    }

    fn visit_while(&mut self, stmt: &While) -> Result<(), ParserError> {
        self.resolve_expr(&stmt.condition);
        self.resolve_stmt(&stmt.body)?;
        Ok(())
    }

    fn visit_class(&mut self, stmt: &Class) -> Result<(), ParserError> {
        let enclosing_class = self.current_class;
        self.current_class = ClassType::Class;

        self.declare(stmt.name.clone())?;
        self.define(stmt.name.clone());

        if let Some(Expr::Variable(sc)) = &stmt.super_class {
            if stmt.name.lexeme.eq(&sc.name.lexeme) {
                crate::error(sc.name.clone(), "A class can't inherit from itself.");
                return Err(ParserError {});
            }
            self.resolve_expr(&Expr::Variable(sc.clone()));
        }

        self.begin_scope();
        self.scopes
            .last_mut()
            .unwrap()
            .insert("this".to_string(), true);

        for method in stmt.methods.iter() {
            if let Stmt::Function(m) = method {
                let declaration = if m.name.lexeme.eq("init") {
                    FunctionType::Initializer
                } else {
                    FunctionType::Method
                };
                self.resolve_function(m, declaration)?;
            }
        }

        self.end_scope();
        self.current_class = enclosing_class;

        Ok(())
    }
}

impl<'a> crate::expr::Visitor<Result<(), ParserError>> for Resolver<'a> {
    fn visit_variable(&mut self, expr: &Variable) -> Result<(), ParserError> {
        if !self.scopes.is_empty()
            && self.scopes.last().unwrap().get(&expr.name.lexeme) == Some(&false)
        {
            crate::error(
                expr.name.clone(),
                "Can't read local variable in its own initializer.",
            );
            return Err(ParserError {});
        }
        self.resolve_local(&Expr::Variable(expr.clone()), expr.name.clone());
        Ok(())
    }

    fn visit_assignment(&mut self, expr: &Assignment) -> Result<(), ParserError> {
        self.resolve_expr(&expr.value);
        self.resolve_local(&Expr::Assignment(expr.clone()), expr.name.clone());
        Ok(())
    }

    fn visit_binary(&mut self, expr: &Binary) -> Result<(), ParserError> {
        self.resolve_expr(&expr.left);
        self.resolve_expr(&expr.right);
        Ok(())
    }

    fn visit_call(&mut self, expr: &Call) -> Result<(), ParserError> {
        self.resolve_expr(&expr.callee);

        for argument in expr.arguments.iter() {
            self.resolve_expr(argument);
        }
        Ok(())
    }

    fn visit_grouping(&mut self, expr: &Grouping) -> Result<(), ParserError> {
        self.resolve_expr(&expr.expr);
        Ok(())
    }

    fn visit_literal(&self, _expr: &Literal) -> Result<(), ParserError> {
        Ok(())
    }

    fn visit_unary(&mut self, expr: &Unary) -> Result<(), ParserError> {
        self.resolve_expr(&expr.right);
        Ok(())
    }

    fn visit_get(&mut self, expr: &Get) -> Result<(), ParserError> {
        self.resolve_expr(&expr.object);
        Ok(())
    }

    fn visit_set(&mut self, expr: &Set) -> Result<(), ParserError> {
        self.resolve_expr(&expr.value);
        self.resolve_expr(&expr.object);
        Ok(())
    }

    fn visit_this(&mut self, expr: &This) -> Result<(), ParserError> {
        if self.current_class == ClassType::None {
            crate::error(expr.keyword.clone(), "Can't use 'this' outside of a class.");
            return Err(ParserError {});
        }

        self.resolve_local(&Expr::This(expr.clone()), expr.keyword.clone());
        Ok(())
    }
}
