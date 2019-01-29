use pest::Parser;

#[derive(Parser)]
#[grammar = "condition.pest"]
pub struct ConditionParser;

enum Term<'a> {
    Eq(&'a str, &'a str),
    Ne(&'a str, &'a str),
    Not(Box<Term<'a>>),
    And(Box<Term<'a>>, Box<Term<'a>>),
    Or(Box<Term<'a>>, Box<Term<'a>>),
}

fn eval_condition(s: &str) -> bool {
    let mut parse_result = ConditionParser::parse(Rule::main, s).unwrap();
    let parse_result = parse_result.next().unwrap();

    let main_result = parse_result.into_inner().next().unwrap();

    use pest::iterators::Pair;

    fn parse_start(pair: Pair<Rule>) -> Term {
        match pair.as_rule() {
            Rule::expr => parse_expr(pair),
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
            Rule::expr => parse_expr(inner_rule),
            _ => unreachable!(),
        }
    }

    fn parse_expr(pair: Pair<Rule>) -> Term {
        let inner_rule = pair.into_inner().next().unwrap();
        match inner_rule.as_rule() {
            Rule::group => parse_group(inner_rule),
            Rule::eq => parse_eq(inner_rule),
            Rule::ne => parse_ne(inner_rule),
            Rule::not => parse_not(inner_rule),
            Rule::and => parse_and(inner_rule),
            Rule::or => parse_or(inner_rule),
            _ => unreachable!(),
        }
    }

    fn parse_operand(pair: Pair<Rule>) -> Term {
        let inner_rule = pair.into_inner().next().unwrap();
        match inner_rule.as_rule() {
            Rule::group => parse_group(inner_rule),
            Rule::eq => parse_eq(inner_rule),
            Rule::ne => parse_ne(inner_rule),
            _ => unreachable!(),
        }
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
            Rule::operand => parse_operand(pair),
            _ => unimplemented!(),
        };
        Term::Not(Box::new(value))
    }

    fn parse_and(pair: Pair<Rule>) -> Term {
        let mut data = pair.into_inner();
        let lhs_pair = data.next().unwrap();
        let lhs = match lhs_pair.as_rule() {
            Rule::operand => parse_operand(lhs_pair),
            _ => unimplemented!(),
        };
        let rhs_pair = data.next().unwrap();
        let rhs = match rhs_pair.as_rule() {
            Rule::operand => parse_operand(rhs_pair),
            _ => unimplemented!(),
        };
        Term::And(Box::new(lhs), Box::new(rhs))
    }

    fn parse_or(pair: Pair<Rule>) -> Term {
        let mut data = pair.into_inner();
        let lhs_pair = data.next().unwrap();
        let lhs = match lhs_pair.as_rule() {
            Rule::operand => parse_operand(lhs_pair),
            _ => unimplemented!(),
        };
        let rhs_pair = data.next().unwrap();
        let rhs = match rhs_pair.as_rule() {
            Rule::operand => parse_operand(rhs_pair),
            _ => unimplemented!(),
        };
        Term::Or(Box::new(lhs), Box::new(rhs))
    }

    let ast = parse_start(main_result);

    fn process<'a>(term: &Term) -> bool {
        match term {
            Term::Eq(lhs, rhs) => lhs == rhs,
            Term::Ne(lhs, rhs) => lhs != rhs,
            Term::Not(value) => !process(&value),
            Term::And(lhs, rhs) => process(&lhs) && process(&rhs),
            Term::Or(lhs, rhs) => process(&lhs) || process(&rhs),
        }
    }

    process(&ast)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq() {
        assert_eq!(eval_condition("s == s"), true);
        assert_eq!(eval_condition("s == q"), false);
        assert_eq!(eval_condition("(s == s)"), true);
        assert_eq!(eval_condition("(s == q)"), false);
        assert_eq!(eval_condition("((s == s))"), true);
        assert_eq!(eval_condition("((s == q))"), false);
        assert_eq!(eval_condition("'hi' == hi"), true);
        assert_eq!(eval_condition("'a b' == 'a b'"), true);
    }

    #[test]
    fn test_ne() {
        assert_eq!(eval_condition("s != s"), false);
        assert_eq!(eval_condition("s != q"), true);
    }

    #[test]
    fn test_not() {
        assert_eq!(eval_condition("!(s == s)"), false);
        assert_eq!(eval_condition("!(s != s)"), true);
    }

    #[test]
    fn test_and() {
        assert_eq!(eval_condition("(s == s) and (q == q)"), true);
        assert_eq!(eval_condition("(s == s) and (a == b)"), false);
    }

    #[test]
    fn test_or() {
        assert_eq!(eval_condition("(s == s) or (q == q)"), true);
        assert_eq!(eval_condition("(s == s) or (a == b)"), true);
    }
}
