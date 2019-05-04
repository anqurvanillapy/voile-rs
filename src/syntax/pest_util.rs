// Tik♂Tok on the clock but the party don't stop!
#[macro_export]
macro_rules! tik_tok {
    () => {
        type Tok<'a> = pest::iterators::Pair<'a, self::Rule>;
        type Tik<'a> = pest::iterators::Pairs<'a, self::Rule>;
    };
}

#[macro_export]
macro_rules! next_rule {
    ($inner:expr, $rule_name:ident) => {{
        let token = $inner.next().unwrap();
        debug_assert_eq!(token.as_rule(), self::Rule::$rule_name);
        $rule_name(token)
    }};
}

#[macro_export]
macro_rules! define_parse_str {
    ($parser:ident, $root_rule:ident, $root_trans:ident, $ret:ty) => {
        pub fn parse_str(input: &str) -> Result<$ret, String> {
            let the_rule: Tok = $parser::parse(Rule::$root_rule, input)
                .map_err(|err| format!("Parse failed at:{}", err))?
                .next()
                .unwrap();
            let end_pos = the_rule.as_span().end_pos().pos();
            let expr = $root_trans(the_rule);
            if end_pos < input.len() {
                let rest = &input[end_pos..];
                Err(format!("Does not consume the following code:\n{}", rest))
            } else {
                Ok(expr)
            }
        }
    };
}

#[inline]
pub fn end_of_rule(inner: &mut Tik) {
    debug_assert_eq!(inner.next(), None)
}
