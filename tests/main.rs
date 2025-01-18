use textfsm_rs::*;

#[cfg(test)]
mod tests {
    use super::*;
    use pest::Parser;

    #[test]
    fn test_regex_pattern() {
        let input = r#"((\d+\/?)+)
"#;
        let pairs = TextFSMParser::parse(Rule::regex_pattern, input).unwrap();
        assert_eq!(pairs.count(), 1);
    }
    #[test]
    fn test_rule_with_err_msg() {
        let input = r#"  ^.* -> Error "Could not parse line:""#;
        let pairs = TextFSMParser::parse(Rule::rule, input).unwrap();
        assert_eq!(pairs.count(), 1);
    }
    #[test]
    fn test_err_msg() {
        let input = r#""test""#;
        let pairs = TextFSMParser::parse(Rule::err_msg, input).unwrap();
        assert_eq!(pairs.count(), 1);
    }
    #[test]
    fn test_value_definition() {
        let input = r#"Value PORT ((\d+\/?)+)
"#;
        let pairs = TextFSMParser::parse(Rule::value_definition, input).unwrap();
        assert_eq!(pairs.count(), 1);
    }

    #[test]
    fn test_state_definition() {
        let input = "Start\n  ^interface -> Continue.Record End\n";
        let pairs = TextFSMParser::parse(Rule::state_definition, input).unwrap();
        assert_eq!(pairs.count(), 1);
    }

    #[test]
    fn test_complete_template() {
        let input = r#"Value Required INTERFACE (.*)
Value DESCRIPTION (.*)

Start
  ^interface -> GetDescription
  ^$ -> Start

GetDescription
  ^description -> Continue.Record Start
  ^$ -> GetDescription
  ^. -> Error
"#;
        let pairs = TextFSMParser::parse(Rule::file, input).unwrap();
        println!("Pairs: {:?}", &pairs);
        assert_eq!(pairs.count(), 3);
    }

    #[test]
    fn test_complete_template_asa() {
        let input = r#"Value Required RESOURCE (.+?)
Value DENIED (\d+)
Value CONTEXT (.+?)

Start
  ^x -> GetDescription

Startr_ecord
  ^x -> X
"#;
        let pairs = TextFSMParser::parse(Rule::file, input).unwrap();
        println!("Pairs: {:?}", &pairs);
        assert_eq!(pairs.count(), 3);
    }

    #[test]
    fn test_complete_template_error() {
        let input = r#"Value PORT_ID (\S+)
Value DESCRIPTION (.+)

Start
  ^=+\s*$$
  ^\s*$$
  ^Port\s+Descriptions\s+on\s\S+\s+\S+\s*$$
  ^Port\s+Id\s+Description\s*$$
  ^${PORT_ID}\s+${DESCRIPTION}\s*$$ -> Record
  ^-+\s*$$
  ^. -> Error"#;
        let pairs = TextFSMParser::parse(Rule::file, input).unwrap();
        println!("Pairs: {:?}", &pairs);
        assert_eq!(pairs.count(), 3);
    }

    #[test]
    fn test_rules_with_no_transitions() {
        let input = r#"Start
  ^interface$
  ^$
"#;
        let pairs = TextFSMParser::parse(Rule::state_definition, input).unwrap();
        println!("Pairs: {:?}", &pairs);
        assert_eq!(pairs.count(), 1);
    }

    #[test]
    fn test_rules_with_no_transitions_complex() {
        let input = r#"Start
  ^PING\s+${DESTINATION}\s+${PKT_SIZE}\s+data\s+bytes*$$
  ^(?:${RESPONSE_STREAM})
  ^\.*$$
  ^\s*$$
  ^-+
  ^${SENT_QTY}\s+packet(?:s)?\s+transmitted,(?:\s+${BOUNCE_QTY}\s+packet(?:s)?\s+bounced,)?\s+${SUCCESS_QTY}\s+packet(?:s)?\s+received,\s+(?:${DUPLICATE_QTY}\s+duplicate(?:s)?)?(?:${LOSS_PCT}%\s+packet\s+loss)?
  ^(?:round-trip\s+min\s+=\s+${RTT_MIN}ms,\s+avg\s+=\s+${RTT_AVG}ms,\s+max\s+=\s+${RTT_MAX}ms,\s+stddev\s+=\s+${STD_DEV}ms)?
  # Error out if raw data does not match any above rules.
  ^.* -> Error "Could not parse line:"
"#;
        let pairs = TextFSMParser::parse(Rule::state_definition, input).unwrap();
        println!("Pairs: {:?}", &pairs);
        assert_eq!(pairs.count(), 1);
    }

    #[test]
    fn test_rules_with_no_transitions_complex_error_nomsg() {
        let input = r#"Start
  ^PING\s+${DESTINATION}\s+${PKT_SIZE}\s+data\s+bytes*$$
  ^(?:${RESPONSE_STREAM})
  ^\.*$$
  ^\s*$$
  ^-+
  ^${SENT_QTY}\s+packet(?:s)?\s+transmitted,(?:\s+${BOUNCE_QTY}\s+packet(?:s)?\s+bounced,)?\s+${SUCCESS_QTY}\s+packet(?:s)?\s+received,\s+(?:${DUPLICATE_QTY}\s+duplicate(?:s)?)?(?:${LOSS_PCT}%\s+packet\s+loss)?
  ^(?:round-trip\s+min\s+=\s+${RTT_MIN}ms,\s+avg\s+=\s+${RTT_AVG}ms,\s+max\s+=\s+${RTT_MAX}ms,\s+stddev\s+=\s+${STD_DEV}ms)?
  # Error out if raw data does not match any above rules.
  ^.* -> Error
"#;
        let pairs = TextFSMParser::parse(Rule::state_definition, input).unwrap();
        println!("Pairs: {:?}", &pairs);
        assert_eq!(pairs.count(), 1);
    }

    #[test]
    fn test_multiple_value_definitions() {
        let input = r#"Value HOSTNAME (.+)
Value VERSION (\d+\.\d+)
Value MODEL (\w+)

"#;
        let pairs = TextFSMParser::parse(Rule::value_definitions, input).unwrap();
        assert_eq!(pairs.count(), 1);
    }

    #[test]
    fn test_complex_rule_nostate() {
        let input = r#"  ^PING\s+${DESTINATION}\s+${PKT_SIZE}\s+data\s+bytes*$$
"#;
        let pairs = TextFSMParser::parse(Rule::rule, input).unwrap();
        assert_eq!(pairs.count(), 1);
    }

    #[test]
    fn test_complex_rule() {
        let input = "  ^interface GigabitEthernet -> Record Start\n";
        let pairs = TextFSMParser::parse(Rule::rule, input).unwrap();
        assert_eq!(pairs.count(), 1);
    }

    #[test]
    fn test_multiple_states() {
        let input = r#"Start
  ^interface -> GetDescription

GetDescription
  ^description -> Record Start
  ^. -> Error
"#;
        let pairs = TextFSMParser::parse(Rule::state_definitions, input).unwrap();
        assert_eq!(pairs.count(), 1);
    }
}

fn print_pair(indent: usize, pair: &Pair<'_, Rule>) {
    // println!("Debug: {:#?}", &pair);
    let spaces = " ".repeat(indent);
    println!("{}Rule:    {:?}", spaces, pair.as_rule());
    println!("{}Span:    {:?}", spaces, pair.as_span());
    println!("{}Text:    {}", spaces, pair.as_str());
    for p in pair.clone().into_inner() {
        print_pair(indent + 2, &p);
    }
}

fn main() {
    for arg in std::env::args().skip(1) {
        // println!("Reading file {}", &arg);
        let template = std::fs::read_to_string(&arg).expect("File read failed");
        let template = format!("{}\n", template);

        match TextFSMParser::parse(Rule::file, &template) {
            Ok(pairs) => {
                for pair in pairs {
                    print_pair(0, &pair);
                }
            }
            Err(e) => panic!("file {} Error: {}", &arg, e),
        }
    }
}
