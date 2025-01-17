use pest::error::Error;
pub use pest::iterators::Pair;
pub use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "varsubst.pest"]
pub struct VariableParser;

#[derive(Debug, PartialEq)]
pub enum ParseChunk {
    DollarDollar,
    Variable(String),
    Text(String),
}

impl VariableParser {
    pub fn parse_dollar_string(input: &str) -> Result<Vec<ParseChunk>, Error<Rule>> {
        let mut out: Vec<ParseChunk> = vec![];
        let pairs = VariableParser::parse(Rule::main, input)?;
        for pair in pairs {
            println!("Rule: {:?}", pair.as_rule());
            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::dollar_dollar => {
                        out.push(ParseChunk::DollarDollar);
                    }
                    Rule::variable_name => {
                        out.push(ParseChunk::Variable(inner_pair.as_str().to_string()));
                    }
                    Rule::literal => {
                        out.push(ParseChunk::Text(inner_pair.as_str().to_string()));
                    }
                    Rule::EOI => {
                        // success
                    }
                    Rule::WHITESPACE => {
                        out.push(ParseChunk::Text(inner_pair.as_str().to_string()));
                    }
                    x => {
                        panic!("Rule {:?} should not happen at varsubst", x);
                    }
                }
                println!(
                    "Inner: {:?} = '{}'",
                    inner_pair.as_rule(),
                    inner_pair.as_str()
                );
            }
        }
        Ok(out)
    }
}
