pub use pest::iterators::Pair;
pub use pest::Parser;
use pest_derive::Parser;
use regex::Regex;

#[derive(Parser)]
#[grammar = "textfsm.pest"]

pub struct TextFSMParser;

pub struct TextFSM {}

impl TextFSM {
    fn print_pair(indent: usize, pair: &Pair<'_, Rule>) {
        // println!("Debug: {:#?}", &pair);
        let spaces = " ".repeat(indent);
        println!("{}Rule:    {:?}", spaces, pair.as_rule());
        println!("{}Span:    {:?}", spaces, pair.as_span());
        println!("{}Text:    {}", spaces, pair.as_str());
        for p in pair.clone().into_inner() {
            Self::print_pair(indent + 2, &p);
        }
    }
    pub fn parse_state_rule(pair: &Pair<'_, Rule>) {
        println!("----- state rule ---");
        let spaces = "";
        for pair in pair.clone().into_inner() {
          println!("{}state Rule:    {:?}", spaces, pair.as_rule());
          println!("{}Span:    {:?}", spaces, pair.as_span());
          println!("{}Text:    {}", spaces, pair.as_str());
        }
    }
    pub fn parse_state_def(pair: &Pair<'_, Rule>) {
        let mut state_name: Option<String> = None;

        for pair in pair.clone().into_inner() {
            if pair.as_rule() == Rule::state_header {
                state_name = Some(pair.as_str().to_string());
            } else if pair.as_rule() == Rule::rules {
                for pair in pair.clone().into_inner() {
                    Self::parse_state_rule(&pair);
                }
            } else {
                let spaces = "";
                println!("{}state def Rule:    {:?}", spaces, pair.as_rule());
                println!("{}Span:    {:?}", spaces, pair.as_span());
                println!("{}Text:    {}", spaces, pair.as_str());
            }
        }
        println!("STATE: {:?}", &state_name);
    }
    pub fn parse_value_def(pair: &Pair<'_, Rule>) {
        for pair in pair.clone().into_inner() {
            if Rule::value_definition == pair.as_rule() {
                println!("value definition");
                let mut name: Option<String> = None;
                let mut regex_pattern: Option<String> = None;
                let mut regex_val: Option<Regex> = None;
                let mut value_options: Option<String> = None;
                for p in pair.clone().into_inner() {
                    match p.as_rule() {
                        Rule::options => value_options = Some(p.as_str().to_string()),
                        Rule::identifier => name = Some(p.as_str().to_string()),
                        Rule::regex_pattern => {
                            regex_val = Regex::new(p.as_str()).ok();
                            regex_pattern = Some(p.as_str().to_string());
                        }
                        x => {
                            panic!("Rule {:?} in value definition", x);
                        }
                    }
                    // Self::print_pair(indent + 2, &p);
                }
                println!(
                    "   {:?} {:?} {:?} [ {:?} ]",
                    &name, &regex_pattern, &regex_val, &value_options
                );
            }
        }
    }
    pub fn process_pair(indent: usize, pair: &Pair<'_, Rule>) {
        // println!("Debug: {:#?}", &pair);
        let spaces = " ".repeat(indent);
        if Rule::value_definitions == pair.as_rule() {
            Self::parse_value_def(&pair);
        } else if Rule::state_definitions == pair.as_rule() {
            for pair in pair.clone().into_inner() {
                println!("=== not last state definition ===");
                Self::parse_state_def(&pair);
            }
        } else if Rule::state_definition == pair.as_rule() {
            println!("=== state definition last ===");
            Self::parse_state_def(&pair);
        } else if Rule::EOI == pair.as_rule() {
        } else {
            println!("{}Rule:    {:?}", spaces, pair.as_rule());
            println!("{}Span:    {:?}", spaces, pair.as_span());
            println!("{}Text:    {}", spaces, pair.as_str());
            for p in pair.clone().into_inner() {
                Self::print_pair(indent + 2, &p);
            }
        }
    }
    pub fn from_file(fname: &str) -> Self {
        println!("Path: {}", &fname);
        let template = std::fs::read_to_string(&fname).expect("File read failed");
        let template = format!("{}\n", template);

        match TextFSMParser::parse(Rule::file, &template) {
            Ok(pairs) => {
                for pair in pairs.clone() {
                    Self::process_pair(0, &pair);
                }
                return TextFSM {};
            }
            Err(e) => panic!("file {} Error: {}", &fname, e),
        }
    }
}
