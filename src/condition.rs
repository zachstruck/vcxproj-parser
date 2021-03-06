use pest::Parser;
use std::path::Path;

#[derive(Parser)]
#[grammar = "condition.pest"]
pub struct ConditionParser;

enum Term<'a> {
    String(&'a str),
    Exprs(Vec<Term<'a>>),
    Exists(&'a str),
    FinalSlash(&'a str),
    Eq(&'a str, &'a str),
    Ne(&'a str, &'a str),
    Not(Box<Term<'a>>),
    And(Box<Term<'a>>),
    Or(Box<Term<'a>>),
}

#[derive(Debug, Clone)]
pub struct ParseError;

pub fn eval_condition(s: &str) -> Result<bool, ParseError> {
    let mut parse_result = ConditionParser::parse(Rule::main, s).map_err(|_| ParseError)?;
    let parse_result = parse_result.next().unwrap();

    let main_result = parse_result.into_inner().next().unwrap();

    use pest::iterators::Pair;

    fn parse_start(pair: Pair<Rule>) -> Term {
        match pair.as_rule() {
            Rule::exprs => parse_exprs(pair),
            _ => unreachable!(),
        }
    }

    fn parse_simple_string(pair: Pair<Rule>) -> &str {
        pair.as_str().trim()
    }

    fn parse_quoted_string(pair: Pair<Rule>) -> &str {
        let quoted = pair.as_str();
        &quoted[1..(quoted.len() - 1)]
    }

    fn parse_string(pair: Pair<Rule>) -> &str {
        let inner_rule = pair.into_inner().next().unwrap();
        match inner_rule.as_rule() {
            Rule::simple_string => parse_simple_string(inner_rule),
            Rule::quoted_string => parse_quoted_string(inner_rule),
            _ => unimplemented!(),
        }
    }

    fn parse_group(pair: Pair<Rule>) -> Term {
        let inner_rule = pair.into_inner().next().unwrap();
        match inner_rule.as_rule() {
            Rule::exprs => parse_exprs(inner_rule),
            _ => unreachable!(),
        }
    }

    fn parse_expr(pair: Pair<Rule>) -> Term {
        let inner_rule = pair.into_inner().next().unwrap();
        match inner_rule.as_rule() {
            Rule::string => Term::String(parse_string(inner_rule)),
            Rule::group => parse_group(inner_rule),
            Rule::exists => parse_exists(inner_rule),
            Rule::final_slash => parse_final_slash(inner_rule),
            Rule::eq => parse_eq(inner_rule),
            Rule::ne => parse_ne(inner_rule),
            Rule::not => parse_not(inner_rule),
            _ => unreachable!(),
        }
    }

    fn parse_exprs(pair: Pair<Rule>) -> Term {
        Term::Exprs(
            pair.into_inner()
                .map(|pair| match pair.as_rule() {
                    Rule::and => parse_and(pair),
                    Rule::or => parse_or(pair),
                    _ => parse_expr(pair),
                })
                .collect(),
        )
    }

    fn parse_exists(pair: Pair<Rule>) -> Term {
        let inner_rule = pair.into_inner().next().unwrap();
        let value = match inner_rule.as_rule() {
            Rule::string => parse_string(inner_rule),
            _ => unreachable!(),
        };
        Term::Exists(value)
    }

    fn parse_final_slash(pair: Pair<Rule>) -> Term {
        let inner_rule = pair.into_inner().next().unwrap();
        let value = match inner_rule.as_rule() {
            Rule::string => parse_string(inner_rule),
            _ => unreachable!(),
        };
        Term::FinalSlash(value)
    }

    fn parse_eq(pair: Pair<Rule>) -> Term {
        let mut data = pair.into_inner();
        let lhs_pair = data.next().unwrap();
        let lhs = match lhs_pair.as_rule() {
            Rule::string => parse_string(lhs_pair),
            _ => unimplemented!(),
        };
        let rhs_pair = data.next().unwrap();
        let rhs = match rhs_pair.as_rule() {
            Rule::string => parse_string(rhs_pair),
            _ => unimplemented!(),
        };
        Term::Eq(lhs, rhs)
    }

    fn parse_ne(pair: Pair<Rule>) -> Term {
        let mut data = pair.into_inner();
        let lhs_pair = data.next().unwrap();
        let lhs = match lhs_pair.as_rule() {
            Rule::string => parse_string(lhs_pair),
            _ => unimplemented!(),
        };
        let rhs_pair = data.next().unwrap();
        let rhs = match rhs_pair.as_rule() {
            Rule::string => parse_string(rhs_pair),
            _ => unimplemented!(),
        };
        Term::Ne(lhs, rhs)
    }

    fn parse_not(pair: Pair<Rule>) -> Term {
        let mut data = pair.into_inner();
        let pair = data.next().unwrap();
        let value = match pair.as_rule() {
            Rule::expr => parse_expr(pair),
            _ => unimplemented!(),
        };
        Term::Not(Box::new(value))
    }

    fn parse_and(pair: Pair<Rule>) -> Term {
        let mut data = pair.into_inner();
        let pair = data.next().unwrap();
        let value = match pair.as_rule() {
            Rule::expr => parse_expr(pair),
            _ => unimplemented!(),
        };
        Term::And(Box::new(value))
    }

    fn parse_or(pair: Pair<Rule>) -> Term {
        let mut data = pair.into_inner();
        let pair = data.next().unwrap();
        let value = match pair.as_rule() {
            Rule::expr => parse_expr(pair),
            _ => unimplemented!(),
        };
        Term::Or(Box::new(value))
    }

    let ast = parse_start(main_result);

    fn process<'a>(term: &Term) -> bool {
        match term {
            Term::String(s) => !s.is_empty(),
            Term::Exprs(exprs) => {
                let mut b = process(&exprs[0]);
                for i in 1..exprs.len() {
                    b = match &exprs[i] {
                        Term::And(rhs) => b && process(&rhs),
                        Term::Or(rhs) => b || process(&rhs),
                        _ => unreachable!(),
                    }
                }
                b
            }
            Term::Exists(path) => Path::new(path).exists(),
            Term::FinalSlash(s) => s.ends_with('\\') || s.ends_with('/'),
            Term::Eq(lhs, rhs) => lhs == rhs,
            Term::Ne(lhs, rhs) => lhs != rhs,
            Term::Not(value) => !process(&value),
            Term::And(rhs) => process(&rhs),
            Term::Or(rhs) => process(&rhs),
        }
    }

    Ok(process(&ast))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval() {
        assert_eq!(eval_condition("'' and ''").unwrap(), false);
        assert_eq!(eval_condition("s == s and s == s").unwrap(), true);
        assert_eq!(eval_condition("s != s and s == s").unwrap(), false);
        assert_eq!(eval_condition("s == s or s == s or s == s").unwrap(), true);
        assert_eq!(eval_condition("'' != '' and !exists('')").unwrap(), false);
    }

    #[test]
    fn test_exists() {
        assert_eq!(eval_condition("!exists('')").unwrap(), true);
    }

    #[test]
    fn test_trailing_slash() {
        assert_eq!(eval_condition("HasTrailingSlash('')").unwrap(), false);
        assert_eq!(eval_condition("HasTrailingSlash('path')").unwrap(), false);
        assert_eq!(eval_condition("HasTrailingSlash('path\\')").unwrap(), true);
        assert_eq!(eval_condition("HasTrailingSlash('path/')").unwrap(), true);
    }

    #[test]
    fn test_eq() {
        assert_eq!(eval_condition("s == s").unwrap(), true);
        assert_eq!(eval_condition("s == q").unwrap(), false);
        assert_eq!(eval_condition("(s == s)").unwrap(), true);
        assert_eq!(eval_condition("(s == q)").unwrap(), false);
        assert_eq!(eval_condition("((s == s))").unwrap(), true);
        assert_eq!(eval_condition("((s == q))").unwrap(), false);
        assert_eq!(eval_condition("'hi' == hi").unwrap(), true);
        assert_eq!(eval_condition("'a b' == 'a b'").unwrap(), true);
    }

    #[test]
    fn test_ne() {
        assert_eq!(eval_condition("s != s").unwrap(), false);
        assert_eq!(eval_condition("s != q").unwrap(), true);
        assert_eq!(eval_condition("'' != ''").unwrap(), false);
    }

    #[test]
    fn test_not() {
        assert_eq!(eval_condition("!(s == s)").unwrap(), false);
        assert_eq!(eval_condition("!(s != s)").unwrap(), true);
    }

    #[test]
    fn test_and() {
        assert_eq!(eval_condition("(s == s) and (q == q)").unwrap(), true);
        assert_eq!(eval_condition("(s == s) and (a == b)").unwrap(), false);
    }

    #[test]
    fn test_or() {
        assert_eq!(eval_condition("(s == s) or (q == q)").unwrap(), true);
        assert_eq!(eval_condition("(s == s) or (a == b)").unwrap(), true);
    }
}
