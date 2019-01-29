use pest::Parser;

#[derive(Parser)]
#[grammar = "condition.pest"]
pub struct ConditionParser;

enum Term<'a> {
    String(&'a str),
    Eq(Box<Term<'a>>, Box<Term<'a>>),
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

    fn parse_simple_string(pair: Pair<Rule>) -> Term {
        Term::String(pair.as_str().trim())
    }

    fn parse_quoted_string(pair: Pair<Rule>) -> Term {
        let quoted = pair.as_str();
        Term::String(&quoted[1..(quoted.len() - 1)])
    }

    fn parse_string(pair: Pair<Rule>) -> Term {
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
            Rule::string => parse_string(inner_rule),
            Rule::group => parse_group(inner_rule),
            Rule::eq => parse_eq(inner_rule),
            _ => unreachable!(),
        }
    }

    fn parse_operand(pair: Pair<Rule>) -> Term {
        let inner_rule = pair.into_inner().next().unwrap();
        match inner_rule.as_rule() {
            Rule::string => parse_string(inner_rule),
            Rule::group => parse_group(inner_rule),
            _ => unreachable!(),
        }
    }

    fn parse_eq(pair: Pair<Rule>) -> Term {
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
        Term::Eq(Box::new(lhs), Box::new(rhs))
    }

    let ast = parse_start(main_result);

    fn process<'a>(term: &Term) -> String {
        match term {
            Term::String(s) => s.to_string(),
            Term::Eq(lhs, rhs) => {
                if process(&*lhs) == process(&*rhs) {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
        }
    }

    process(&ast) == "true"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_string() {
        assert_eq!(eval_condition("true"), true);
        assert_eq!(eval_condition("false"), false);
    }

    #[test]
    fn test_eq() {
        assert_eq!(eval_condition("s == s"), true);
        assert_eq!(eval_condition("s == q"), false);
        assert_eq!(eval_condition("(s == s)"), true);
        assert_eq!(eval_condition("(s == q)"), false);
        assert_eq!(eval_condition("((s == s))"), true);
        assert_eq!(eval_condition("((s == q))"), false);
        assert_eq!(eval_condition("(s == s) == (s == s)"), true);

        assert_eq!(eval_condition("'hi' == hi"), true);
        assert_eq!(eval_condition("'a b' == 'a b'"), true);
    }
}
