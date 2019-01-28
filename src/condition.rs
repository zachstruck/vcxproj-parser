use pest::Parser;

#[derive(Parser)]
#[grammar = "condition.pest"]
pub struct ConditionParser;

fn eval_condition(s: &str) -> bool {
    let parse_result = ConditionParser::parse(Rule::simple_string, s);
    dbg!(&parse_result);
    parse_result.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval() {
        assert_eq!(eval_condition("true"), true);
    }
}
