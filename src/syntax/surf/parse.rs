use crate::syntax::common::{Level, SyntaxInfo};
use crate::syntax::surf::{Declaration, Expression, Identifier, NamedExpression};
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "syntax/surf/grammar.pest"]
/// The name stands for "Voile's Parser"
struct VoileParser;

// Tik♂Tok on the clock but the party don't stop!
type Tok<'a> = Pair<'a, Rule>;
type Tik<'a> = Pairs<'a, Rule>;

/// Parse a string into an optional expr based on `file` rule:
/// ```ignore
/// file = { WHITESPACE* ~ expr }
/// ```
pub fn parse_str(input: &str) -> Result<Vec<Declaration>, String> {
    let the_rule: Tok = VoileParser::parse(Rule::file, input)
        .map_err(|err| format!("Parse failed at:{}", err))?
        .next()
        .unwrap();
    let end_pos = the_rule.as_span().end_pos().pos();
    let expr = declarations(the_rule);
    if end_pos < input.len() {
        let rest = &input[end_pos..];
        Err(format!("Does not consume the following code:\n{}", rest))
    } else {
        Ok(expr)
    }
}

macro_rules! next_rule {
    ($inner:expr, $rule_name:ident, $function:ident) => {{
        let token = $inner.next().unwrap();
        debug_assert_eq!(token.as_rule(), Rule::$rule_name);
        $function(token)
    }};
}

macro_rules! expr_parser {
    ($name:ident,$smaller:ident,$cons:ident) => {
        fn $name(rules: Tok) -> Expr {
            let the_rule: Tok = rules.into_inner().next().unwrap();
            let mut exprs: Vec<Expr> = Default::default();
            for smaller in the_rule.into_inner() {
                exprs.push($smaller(smaller));
            }
            Expr::$cons(exprs)
        }
    };
}

#[inline]
fn next_identifier(inner: &mut Tik) -> Identifier {
    next_rule!(inner, identifier, identifier)
}

#[inline]
fn next_expr(inner: &mut Tik) -> Expression {
    next_rule!(inner, expr, expr)
}

#[inline]
fn end_of_rule(inner: &mut Tik) {
    debug_assert_eq!(inner.next(), None)
}

fn declarations(the_rule: Tok) -> Vec<Declaration> {
    let mut decls: Vec<Declaration> = Default::default();
    for prefix_parameter in the_rule.into_inner() {
        decls.push(declaration(prefix_parameter));
    }
    decls
}

fn declaration(rules: Tok) -> Declaration {
    let the_rule: Tok = rules.into_inner().next().unwrap();
    match the_rule.as_rule() {
        Rule::signature => Declaration::Sign(named_expr(the_rule)),
        Rule::implementation => Declaration::Impl(named_expr(the_rule)),
        _ => unreachable!(),
    }
}

fn named_expr(rules: Tok) -> NamedExpression {
    let mut inner: Tik = rules.into_inner();
    let identifier = next_identifier(&mut inner);
    let expr = next_expr(&mut inner);
    end_of_rule(&mut inner);
    NamedExpression {
        name: identifier,
        body: expr,
    }
}

expr_parser!(dollar_expr, pipe_expr, Pipe);
expr_parser!(pipe_expr, sum_expr, Pipe);
expr_parser!(sum_expr, app_expr, Pipe);
expr_parser!(app_expr, primary_expr, Pipe);

fn expr(rules: Tok) -> Expression {
    let the_rule: Tok = rules.into_inner().next().unwrap();
    match the_rule.as_rule() {
        Rule::dollar_expr => dollar_expr(the_rule),
        Rule::pipe_expr => pipe_expr(the_rule),
        Rule::sum_expr => sum_expr(the_rule),
        Rule::app_expr => app_expr(the_rule),
        Rule::primary_expr => primary_expr(the_rule),
        e => panic!("Unexpected rule: {:?} with token {}", e, the_rule.as_str()),
    }
}

fn primary_expr(rules: Tok) -> Expression {
    let the_rule: Tok = rules.into_inner().next().unwrap();
    match the_rule.as_rule() {
        Rule::identifier => Expression::Var(From::from(the_rule.as_span())),
        Rule::type_keyword => type_keyword(the_rule),
        Rule::expr => expr(the_rule),
        e => panic!("Unexpected rule: {:?} with token {}", e, the_rule.as_str()),
    }
}

fn type_keyword(rules: Tok) -> Expression {
    let syntax_info: SyntaxInfo = From::from(rules.as_span());
    let mut inner: Tik = rules.into_inner();
    let level: Level = inner.next().unwrap().as_str().parse().unwrap();
    end_of_rule(&mut inner);
    Expression::Type(syntax_info, level)
}

fn identifier(rule: Tok) -> Identifier {
    Identifier {
        info: From::from(rule.as_span()),
    }
}

#[cfg(test)]
mod tests {
    use super::super::parse_str_err_printed;

    #[test]
    fn simple_declaration_parsing() {
        parse_str_err_printed("val a : b;").unwrap();
        parse_str_err_printed("val a : b").unwrap_err();
        parse_str_err_printed("a : b").unwrap_err();
        parse_str_err_printed("let a = b;").unwrap();
        parse_str_err_printed("a = b").unwrap_err();
    }

    #[test]
    fn simple_expr_parsing() {
        parse_str_err_printed("let a = Type233;").unwrap();
    }
}
