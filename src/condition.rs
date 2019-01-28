use pest::Parser;

#[derive(Parser)]
#[grammar = "condition.pest"]
pub struct ConditionParser;

enum Term<'a> {
    Eq(&'a str, &'a str),
}

fn eval_condition(s: &str) -> bool {
    let mut parse_result = ConditionParser::parse(Rule::main, s).unwrap();
    let parse_result = parse_result.next().unwrap();

    let main_result = parse_result.into_inner().next().unwrap();

    use pest::iterators::Pair;

    fn parse_start(pair: Pair<Rule>) -> Term {
        match pair.as_rule() {
            Rule::eq => parse_eq(pair),
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

    match ast {
        Term::Eq(lhs, rhs) => lhs == rhs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq() {
        assert_eq!(eval_condition("s == s"), true);
        assert_eq!(eval_condition("s == q"), false);
    }
}
