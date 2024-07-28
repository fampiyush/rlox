use crate::expr::*;
use crate::token::LiteralTypes;
pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&self, expr: &Expr) -> String {
        expr.accept(self)
    }

    fn parenthesize(&self, name: &str, exprs: &[&Expr]) -> String {
        let mut res = String::new();
        res.push_str(&format!("({}", name));
        for expr in exprs.iter() {
            res.push(' ');
            let s = expr.accept(self);
            res.push_str(&s);
        }
        res.push(')');

        res
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_binary(&self, expr: &Binary) -> String {
        self.parenthesize(&expr.operator.lexeme, &[&expr.left, &expr.right])
    }

    fn visit_grouping(&self, expr: &Grouping) -> String {
        self.parenthesize("group", &[&expr.expr])
    }

    fn visit_literal(&self, expr: &Literal) -> String {
        match &expr.value {
            LiteralTypes::String(val) => val.to_string(),
            LiteralTypes::Number(val) => val.to_string(),
            LiteralTypes::Bool(val) => val.to_string(),
            LiteralTypes::Nil => "nil".to_string(),
        }
    }

    fn visit_unary(&self, expr: &Unary) -> String {
        self.parenthesize(&expr.operator.lexeme, &[&expr.right])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::*;

    fn example() -> String {
        let expression = Expr::Binary(Binary {
            left: Box::new(Expr::Unary(Unary {
                operator: Token::new(TokenType::Minus, "-".to_string(), LiteralTypes::Nil, 1),
                right: Box::new(Expr::Literal(Literal {
                    value: LiteralTypes::Number(123.0),
                })),
            })),
            operator: Token::new(TokenType::Star, "*".to_string(), LiteralTypes::Nil, 1),
            right: Box::new(Expr::Grouping(Grouping {
                expr: Box::new(Expr::Literal(Literal {
                    value: LiteralTypes::Number(45.67),
                })),
            })),
        });

        AstPrinter.print(&expression)
    }

    #[test]
    fn ast_printer_test() {
        let s = example();
        assert_eq!(s, "(* (- 123) (group 45.67))");
    }
}
