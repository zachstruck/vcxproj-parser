use pest::Parser;

#[derive(Parser)]
#[grammar = "condition.pest"]
pub struct ConditionParser;

enum Term<'a> {
    Group(Box<Term<'a>>),
    Eq(&'a str, &'a str),
}

fn eval_condition(s: &str) -> bool {
    let mut parse_result = ConditionParser::parse(Rule::main, s).unwrap();
    let parse_result = parse_result.next().unwrap();

    let main_result = parse_result.into_inner().next().unwrap();

    use pest::iterators::Pair;

    fn parse_start(pair: Pair<Rule>) -> Term {
        match pair.as_rule() {
            Rule::term => parse_term(pair),
            _ => unreachable!(),
        }
    }

    fn parse_group(pair: Pair<Rule>) -> Term {
        let inner_rule = pair.into_inner().next().unwrap();
        match inner_rule.as_rule() {
            Rule::term => parse_term(inner_rule),
            _ => unreachable!(),
        }
    }

    fn parse_term(pair: Pair<Rule>) -> Term {
        let inner_rule = pair.into_inner().next().unwrap();
        match inner_rule.as_rule() {
            Rule::group => parse_group(inner_rule),
            Rule::eq => parse_eq(inner_rule),
            _ => unreachable!(),
        }
    }

    fn parse_eq(pair: Pair<Rule>) -> Term {
        let mut data = pair.into_inner();
        let lhs_pair = data.next().unwrap();
        let lhs = match lhs_pair.as_rule() {
            Rule::simple_string => lhs_pair.as_str().trim(),
            _ => unimplemented!(),
        };
        let rhs_pair = data.next().unwrap();
        let rhs = match rhs_pair.as_rule() {
            Rule::simple_string => rhs_pair.as_str().trim(),
            _ => unimplemented!(),
        };
        Term::Eq(lhs, rhs)
    }

    let ast = parse_start(main_result);

    fn process<'a>(term: &Term) -> bool {
        match term {
            Term::Group(inner) => process(&*inner),
            Term::Eq(lhs, rhs) => lhs == rhs,
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
    }
}
