pub use pest::iterators::Pair;
pub use pest::Parser;
use pest_derive::Parser;

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
    fn from_file(path: &str) -> Self {
        let template = std::fs::read_to_string(&path).expect("File read failed");
        let template = format!("{}\n", template);

        match TextFSMParser::parse(Rule::file, &template) {
            Ok(pairs) => {
                for pair in pairs.clone() {
                    Self::print_pair(0, &pair);
                }
                return TextFSM {};
            }
            Err(e) => panic!("file {} Error: {}", &path, e),
        }
    }
}
