pub use pest::iterators::Pair;
pub use pest::Parser;
use pest_derive::Parser;
use regex::Regex;
use std::collections::HashMap;

pub mod varsubst;

#[derive(Parser)]
#[grammar = "textfsm.pest"]
pub struct TextFSMParser;

pub struct TextFSM {}

#[derive(Debug, Default, PartialEq)]
pub enum LineAction {
    #[default]
    Continue,
    Next,
}

#[derive(Debug, Default, PartialEq)]
pub enum RecordAction {
    #[default]
    NoRecord,
    Record,
    Clear,
    Clearall,
}

#[derive(Debug, PartialEq)]
pub enum NextState {
    Error(Option<String>),
    NamedState(String),
}

#[derive(Debug, Default, PartialEq)]
pub struct RuleTransition {
    line_action: LineAction,
    record_action: RecordAction,
    maybe_next_state: Option<NextState>,
}

#[derive(Debug, Default, PartialEq)]
pub struct StateRule {
    rule_match: String,
    transition: RuleTransition,
}

#[derive(Debug, Default, PartialEq)]
pub struct ValueDefinition {
    name: String,
    regex_pattern: String,
    options: Option<String>,
}

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
    pub fn parse_state_rule_transition(pair: &Pair<'_, Rule>) -> RuleTransition {
        let mut record_action: RecordAction = Default::default();
        let mut line_action: LineAction = Default::default();
        let mut maybe_next_state: Option<NextState> = None;
        // Self::print_pair(5, pair);
        for pair in pair.clone().into_inner() {
            match pair.as_rule() {
                Rule::record_action => {
                    record_action = match pair.as_str() {
                        "Record" => RecordAction::Record,
                        "NoRecord" => RecordAction::NoRecord,
                        "Clear" => RecordAction::Clear,
                        "Clearall" => RecordAction::Clearall,
                        x => panic!("Record action {} not supported", x),
                    };
                }
                Rule::line_action => {
                    line_action = match pair.as_str() {
                        "Continue" => LineAction::Continue,
                        "Next" => LineAction::Next,
                        x => panic!("Record action {} not supported", x),
                    };
                }
                Rule::err_state => {
                    let mut maybe_err_msg: Option<String> = None;
                    for p in pair.clone().into_inner() {
                        if p.as_rule() == Rule::err_msg {
                            maybe_err_msg = Some(p.as_str().to_string());
                        }
                    }
                    maybe_next_state = Some(NextState::Error(maybe_err_msg));
                }
                Rule::next_state => {
                    maybe_next_state = Some(NextState::NamedState(pair.as_str().to_string()));
                }
                x => {
                    panic!("Rule {:?} not supported!", &x);
                }
            }
        }
        RuleTransition {
            record_action,
            line_action,
            maybe_next_state,
        }
    }
    pub fn parse_state_rule(pair: &Pair<'_, Rule>) -> StateRule {
        let mut rule_match: Option<String> = None;
        // println!("----- state rule ---");
        // Self::print_pair(10, pair);
        // println!("--------");
        let mut transition: RuleTransition = Default::default();
        let spaces = "";
        for pair in pair.clone().into_inner() {
            match pair.as_rule() {
                Rule::rule_match => {
                    rule_match = Some(pair.as_str().to_string());
                }
                Rule::transition_action => {
                    transition = Self::parse_state_rule_transition(&pair);
                    // println!("TRANSITION: {:?}", &transition);
                }
                x => {
                    println!("{}state Rule:    {:?}", spaces, pair.as_rule());
                    println!("{}Span:    {:?}", spaces, pair.as_span());
                    println!("{}Text:    {}", spaces, pair.as_str());
                    panic!("state rule {:?} not supported", &x);
                }
            }
        }
        let rule_match = rule_match.expect("rule_match must be always set");
        StateRule {
            rule_match,
            transition,
        }
    }
    pub fn parse_state_definition(
        pair: &Pair<'_, Rule>,
        values: &HashMap<String, ValueDefinition>,
    ) {
        let mut state_name: Option<String> = None;
        // Self::print_pair(20, pair);

        for pair in pair.clone().into_inner() {
            match pair.as_rule() {
                Rule::state_header => {
                    state_name = Some(pair.as_str().to_string());
                    // println!("SET STATE NAME: {:?}", &state_name);
                }
                Rule::rules => {
                    for pair in pair.clone().into_inner() {
                        let rule = Self::parse_state_rule(&pair);
                        println!("RULE: {:#?}", &rule);
                        let varsubst =
                            varsubst::VariableParser::parse_dollar_string(&rule.rule_match)
                                .unwrap();
                        println!("DOLLAR STR: {:?}", &varsubst);
                    }
                }
                x => {
                    let spaces = "";
                    println!("{}state def Rule:    {:?}", spaces, pair.as_rule());
                    println!("{}Span:    {:?}", spaces, pair.as_span());
                    println!("{}Text:    {}", spaces, pair.as_str());
                    panic!("Rule not supported in state definition: {:?}", &x);
                }
            }
        }
        println!("STATE: {:?}", &state_name);
    }
    pub fn parse_state_defs(pair: &Pair<'_, Rule>, values: &HashMap<String, ValueDefinition>) {
        println!("=== STATE DEFINITIONS ===");
        for pair in pair.clone().into_inner() {
            match pair.as_rule() {
                Rule::state_definition => {
                    let state = Self::parse_state_definition(&pair, values);
                }
                x => {
                    panic!("state definition rule {:?} not supported", x);
                }
            }
        }
    }
    pub fn parse_value_definition(pair: &Pair<'_, Rule>) -> Result<ValueDefinition, String> {
        // println!("value definition");
        let mut name: Option<String> = None;
        let mut regex_pattern: Option<String> = None;
        let mut regex_val: Option<Regex> = None;
        let mut options: Option<String> = None;
        for p in pair.clone().into_inner() {
            match p.as_rule() {
                Rule::options => options = Some(p.as_str().to_string()),
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
        if let (Some(name), Some(regex_pattern)) = (name.clone(), regex_pattern.clone()) {
            Ok(ValueDefinition {
                name,
                regex_pattern,
                options,
            })
        } else {
            Err(format!(
                "Error parsing value: {:?} {:?} {:?} [ {:?} ]",
                &name, &regex_pattern, &regex_val, &options
            ))
        }
    }
    pub fn parse_value_defs(
        pair: &Pair<'_, Rule>,
    ) -> Result<HashMap<String, ValueDefinition>, String> {
        let mut vals = HashMap::new();
        for pair in pair.clone().into_inner() {
            if Rule::value_definition == pair.as_rule() {
                let val = Self::parse_value_definition(&pair)?;
                vals.insert(val.name.clone(), val);
            }
        }
        Ok(vals)
    }
    pub fn from_file(fname: &str) -> Self {
        println!("Path: {}", &fname);
        let template = std::fs::read_to_string(&fname).expect("File read failed");
        // pad with a newline, because dealing with a missing one within grammar is a PITA
        let template = format!("{}\n", template);

        let mut seen_eoi = false;
        let mut values: HashMap<String, ValueDefinition> = HashMap::new();

        match TextFSMParser::parse(Rule::file, &template) {
            Ok(pairs) => {
                for pair in pairs.clone() {
                    match pair.as_rule() {
                        Rule::value_definitions => {
                            values = Self::parse_value_defs(&pair).unwrap();
                        }
                        Rule::state_definitions => {
                            Self::parse_state_defs(&pair, &values);
                        }
                        Rule::EOI => {
                            seen_eoi = true;
                        }
                        x => {
                            panic!("RULE {:?} not supported", &x);
                        }
                    }
                    // Self::process_pair(0, &pair);
                }
                return TextFSM {};
            }
            Err(e) => panic!("file {} Error: {}", &fname, e),
        }
    }
}
