use pest::Parser;
use pest_derive::Parser;

use textfsm_rs::varsubst::*;

#[cfg(test)]
mod tests {
    use super::*;
    use pest::error::Error;
    use pest::Parser;

    fn parse_test(input: &str) -> Result<(), Error<Rule>> {
        let pairs = VariableParser::parse(Rule::main, input)?;
        for pair in pairs {
            println!("Rule: {:?}", pair.as_rule());
            for inner_pair in pair.into_inner() {
                println!(
                    "Inner: {:?} = '{}'",
                    inner_pair.as_rule(),
                    inner_pair.as_str()
                );
            }
        }
        Ok(())
    }

    #[test]
    fn test_simple_variable() -> Result<(), Error<Rule>> {
        parse_test("$simple_var")
    }

    #[test]
    fn test_braced_variable() -> Result<(), Error<Rule>> {
        parse_test("${braced_var}")
    }

    #[test]
    fn test_double_dollar() -> Result<(), Error<Rule>> {
        parse_test("$$")
    }

    #[test]
    fn test_mixed_text_and_vars() -> Result<(), Error<Rule>> {
        parse_test("Hello ${name}, your ID is $id!")
    }

    #[test]
    fn test_multiple_vars_in_path() -> Result<(), Error<Rule>> {
        parse_test("/path/${dir}/$$/$subdir/file")
    }

    #[test]
    fn test_adjacent_vars() -> Result<(), Error<Rule>> {
        parse_test("${var1}$var2$${var3}$$")
    }

    #[test]
    fn test_vars_with_underscores() -> Result<(), Error<Rule>> {
        parse_test("${my_var}_$another_var")
    }

    #[test]
    fn test_complex_path() -> Result<(), Error<Rule>> {
        parse_test("/base/$${project_name}/${env}_${region}/config")
    }

    #[test]
    fn test_empty_string() -> Result<(), Error<Rule>> {
        parse_test("")
    }

    #[test]
    fn test_only_literal() -> Result<(), Error<Rule>> {
        parse_test("Just a regular string without variables")
    }

    #[test]
    fn test_whitespace_handling() -> Result<(), Error<Rule>> {
        parse_test("${var1}   $var2\n${var3}\t$$")
    }

    #[test]
    #[should_panic]
    fn test_unclosed_brace() {
        parse_test("${unclosed").unwrap();
    }

    #[test]
    fn test_dollar_in_literal() -> Result<(), Error<Rule>> {
        parse_test("Cost: 100$ (${amount})")
    }

    #[test]
    fn test_complex_nested_path() -> Result<(), Error<Rule>> {
        parse_test("/data/${env}/$${service_name}/${region}/v${version}/$type/$$")
    }
}
