use log::{debug, error, info, log_enabled, trace, Level};
pub use pest::iterators::Pair;
pub use pest::Parser;
use pest_derive::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod varsubst;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataRecord(pub HashMap<String, Value>);

impl DataRecord {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn overwrite_from(&mut self, from: DataRecord) {
        for (k, v) in from.0 {
            self.0.insert(k, v);
        }
    }
    pub fn compare_sets(
        result: &Vec<Self>,
        other: &Vec<Self>,
    ) -> (Vec<Vec<String>>, Vec<Vec<String>>) {
        let mut only_in_result: Vec<Vec<String>> = vec![];
        let mut only_in_other: Vec<Vec<String>> = vec![];

        for (i, irec) in result.iter().enumerate() {
            let mut vo: Vec<String> = vec![];
            for (k, v) in &irec.0 {
                if i < other.len() {
                    let v0 = other[i].get(k);
                    if v0.is_none() || v0.unwrap() != v {
                        vo.push(k.clone());
                    }
                } else {
                    vo.push(k.clone());
                }
            }
            only_in_result.push(vo);
        }

        for (i, irec) in other.iter().enumerate() {
            let mut vo: Vec<String> = vec![];
            for (k, v) in &irec.0 {
                if i < result.len() {
                    let v0 = result[i].get(k);
                    if v0.is_none() || v0.unwrap() != v {
                        vo.push(k.clone());
                    }
                } else {
                    vo.push(k.clone());
                }
            }
            only_in_other.push(vo);
        }
        (only_in_result, only_in_other)
    }

    pub fn insert(&mut self, name: String, value: String) {
        if self.0.contains_key(&name) {
            let mut existing = self.0.remove(&name);
            match existing {
                None => {
                    panic!("internal error");
                }
                Some(Value::Single(oldval)) => {
                    let newval = Value::List(vec![oldval, value]);
                    self.0.insert(name, newval);
                }
                Some(Value::List(ref mut oldlist)) => {
                    oldlist.push(value);
                    self.0.insert(name, existing.unwrap());
                }
            }
        } else {
            self.0.insert(name, Value::Single(value));
        }
    }

    pub fn append_value(&mut self, name: String, value: Value) {
        if self.0.contains_key(&name) {
            let mut existing = self.0.remove(&name);
            match existing {
                None => {
                    panic!("internal error");
                }
                Some(Value::Single(oldval)) => match value {
                    Value::Single(val) => {
                        let newval = Value::Single(val);
                        self.0.insert(name, newval);
                    }
                    Value::List(lst) => {
                        panic!(
                            "can not append list {:?} to single {:?} in var {}",
                            &lst, &oldval, &name
                        );
                    }
                },
                Some(Value::List(ref mut oldlist)) => match value {
                    Value::Single(val) => {
                        oldlist.push(val);
                        self.0.insert(name, existing.unwrap());
                    }
                    Value::List(mut lst) => {
                        oldlist.append(&mut lst);
                        self.0.insert(name, existing.unwrap());
                    }
                },
            }
        } else {
            self.0.insert(name, value);
        }
    }

    pub fn remove(&mut self, key: &str) {
        self.0.remove(key);
    }
    pub fn keys(&self) -> std::collections::hash_map::Keys<'_, String, Value> {
        self.0.keys()
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, String, Value> {
        self.0.iter()
    }
}
impl Default for DataRecord {
    fn default() -> Self {
        DataRecord(Default::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Value {
    Single(String),
    List(Vec<String>),
}

#[derive(Parser, Debug, Default, Clone)]
#[grammar = "textfsm.pest"]
pub struct TextFSMParser {
    pub values: HashMap<String, ValueDefinition>,
    pub mandatory_values: Vec<String>,
    pub states: HashMap<String, StateCompiled>,
}

#[derive(Debug, Default, Clone)]
pub struct TextFSM {
    pub parser: TextFSMParser,
    pub curr_state: String,
    pub curr_record: DataRecord,
    pub filldown_record: DataRecord,
    pub records: Vec<DataRecord>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LineAction {
    Continue,
    Next(Option<NextState>),
}

impl Default for LineAction {
    fn default() -> LineAction {
        LineAction::Next(None)
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub enum RecordAction {
    #[default]
    NoRecord,
    Record,
    Clear,
    Clearall,
}

#[derive(Debug, PartialEq, Clone)]
pub enum NextState {
    Error(Option<String>),
    NamedState(String),
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct RuleTransition {
    line_action: LineAction,
    record_action: RecordAction,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct StateRule {
    rule_match: String,
    transition: RuleTransition,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ValueDefinition {
    name: String,
    is_filldown: bool,
    is_key: bool,
    is_required: bool,
    is_list: bool,
    is_fillup: bool,
    regex_pattern: String,
    options: Option<String>,
}

#[derive(Debug, Clone)]
pub enum MultiRegex {
    Classic(regex::Regex),
    Fancy(fancy_regex::Regex),
}

#[derive(Debug, Clone)]
pub struct StateRuleCompiled {
    _rule_match: String,
    _expanded_rule_match: String,
    match_variables: Vec<String>,
    maybe_regex: Option<MultiRegex>,
    transition: RuleTransition,
}

#[derive(Debug, Clone)]
pub struct StateCompiled {
    name: String,
    rules: Vec<StateRuleCompiled>,
}

#[derive(Debug, Clone)]
pub enum DataRecordConversion {
    LowercaseKeys,
}

impl TextFSMParser {
    fn _log_pair(indent: usize, pair: &Pair<'_, Rule>) {
        // println!("Debug: {:#?}", &pair);
        let spaces = " ".repeat(indent);
        trace!("{}Rule:    {:?}", spaces, pair.as_rule());
        trace!("{}Span:    {:?}", spaces, pair.as_span());
        trace!("{}Text:    {}", spaces, pair.as_str());
        for p in pair.clone().into_inner() {
            Self::_log_pair(indent + 2, &p);
        }
    }
    pub fn parse_state_rule_transition(pair: &Pair<'_, Rule>) -> RuleTransition {
        let mut record_action: RecordAction = Default::default();
        let mut line_action: LineAction = Default::default();
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
                        "Next" => LineAction::Next(None),
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
                    let next_state = NextState::Error(maybe_err_msg);
                    line_action = LineAction::Next(Some(next_state));
                }
                Rule::next_state => {
                    if line_action == LineAction::Next(None) {
                        let next_state = NextState::NamedState(pair.as_str().to_string());
                        line_action = LineAction::Next(Some(next_state));
                    } else {
                        panic!(
                            "Line action {:?} does not support next state (attempted {:?})",
                            &line_action,
                            pair.as_str()
                        );
                    }
                }
                x => {
                    panic!("Rule {:?} not supported!", &x);
                }
            }
        }
        RuleTransition {
            record_action,
            line_action,
        }
    }
    pub fn parse_state_rule(pair: &Pair<'_, Rule>) -> StateRule {
        let mut rule_match: Option<String> = None;
        // println!("----- state rule ---");
        // Self::print_pair(10, pair);
        // println!("--------");
        let mut transition: RuleTransition = Default::default();
        let mut has_action = false;
        let spaces = "";
        for pair in pair.clone().into_inner() {
            match pair.as_rule() {
                Rule::rule_match => {
                    rule_match = Some(pair.as_str().to_string());
                }
                Rule::transition_action => {
                    has_action = true;
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
        let mut rule_match = rule_match.expect("rule_match must be always set");
        if (rule_match.ends_with(" ") || rule_match.ends_with("\t")) && !has_action {
            println!(
                "WARNING: '{}' has trailing spaces without transition action!",
                &rule_match
            );
            rule_match = rule_match.trim_end().to_string();
        }
        if rule_match.contains(r#"\<"#) {
            println!("WARNING: replacing \\< with < in '{}'", &rule_match);
            rule_match = rule_match.replace("\\<", "<");
        }
        if rule_match.contains(r#"\>"#) {
            println!("WARNING: replacing \\> with > in '{}'", &rule_match);
            rule_match = rule_match.replace("\\>", ">");
        }
        StateRule {
            rule_match,
            transition,
        }
    }

    pub fn compile_state_rule(
        rule: &StateRule,
        values: &HashMap<String, ValueDefinition>,
    ) -> Result<StateRuleCompiled, String> {
        let mut expanded_rule_match: String = format!("");
        let rule_match = rule.rule_match.clone();
        let mut match_variables: Vec<String> = vec![];
        let varsubst = varsubst::VariableParser::parse_dollar_string(&rule_match).unwrap();
        // println!("DOLLAR STR: {:?}", &varsubst);
        {
            use varsubst::ParseChunk;
            for i in &varsubst {
                match i {
                    ParseChunk::DollarDollar => expanded_rule_match.push('$'),
                    ParseChunk::Text(s) => expanded_rule_match.push_str(s),
                    ParseChunk::Variable(v) => match values.get(v) {
                        Some(val) => {
                            let v_out = format!("(?P<{}>{})", v, val.regex_pattern);
                            expanded_rule_match.push_str(&v_out);
                            match_variables.push(v.to_string());
                        }
                        None => panic!(
                            "Can not find variable '{}' while parsing rule_match '{}'",
                            &v, &rule.rule_match
                        ),
                    },
                }
            }
        }
        // println!("OUT_STR: {}", expanded_rule_match);

        let regex_val = match Regex::new(&expanded_rule_match) {
            Ok(r) => MultiRegex::Classic(r),
            Err(_e) => {
                use fancy_regex::Error;
                use fancy_regex::ParseError;

                let freg = loop {
                    let fancy_regex = fancy_regex::Regex::new(&expanded_rule_match);
                    match fancy_regex {
                        Ok(x) => {
                            break x;
                        }
                        Err(Error::ParseError(pos, e)) => {
                            println!("STR:{}", &expanded_rule_match[0..pos + 1]);
                            println!("ERR:{}^", " ".repeat(pos));
                            match e {
                                ParseError::TargetNotRepeatable => {
                                    if let Some(char_index) =
                                        expanded_rule_match.char_indices().nth(pos)
                                    {
                                        println!("WARNING: repeat quantifier on a lookahead, lookbehind or other zero-width item");
                                        expanded_rule_match.remove(char_index.0);
                                    } else {
                                        panic!("Can not fix up regex!");
                                    }
                                }
                                e => {
                                    panic!("Error: {:?}", &e);
                                }
                            }
                        }
                        x => {
                            panic!("Error: {:?}", &x);
                        }
                    }
                };
                MultiRegex::Fancy(freg)
            }
        };
        let maybe_regex = Some(regex_val);
        let transition = rule.transition.clone();
        let _rule_match = rule_match;
        let _expanded_rule_match = expanded_rule_match;

        Ok(StateRuleCompiled {
            _rule_match,
            _expanded_rule_match,
            match_variables,
            maybe_regex,
            transition,
        })
    }
    pub fn parse_and_compile_state_definition(
        pair: &Pair<'_, Rule>,
        values: &HashMap<String, ValueDefinition>,
    ) -> Result<StateCompiled, String> {
        let mut name: Option<String> = None;
        // Self::print_pair(20, pair);
        let mut rules: Vec<StateRuleCompiled> = vec![];

        for pair in pair.clone().into_inner() {
            match pair.as_rule() {
                Rule::state_header => {
                    name = Some(pair.as_str().to_string());
                    // println!("SET STATE NAME: {:?}", &state_name);
                }
                Rule::rules => {
                    for pair in pair.clone().into_inner() {
                        let rule = Self::parse_state_rule(&pair);
                        trace!("PARSED RULE [{:?}]: {:#?}", &name, &rule);
                        let compiled_rule = Self::compile_state_rule(&rule, values).unwrap();
                        rules.push(compiled_rule);
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
        let name = name.expect("internal error - state must have a name");
        Ok(StateCompiled { name, rules })
    }
    /*
    pub fn parse_state_defs(pair: &Pair<'_, Rule>, values: &HashMap<String, ValueDefinition>) {
        // println!("=== STATE DEFINITIONS ===");
        for pair in pair.clone().into_inner() {
            match pair.as_rule() {
                Rule::state_definition => {
                    let state = Self::parse_and_compile_state_definition(&pair, values).unwrap();
                    // println!("Compiled state: {:#?}", &state);
                }
                x => {
                    panic!("state definition rule {:?} not supported", x);
                }
            }
        }
    }
    */
    pub fn parse_value_definition(pair: &Pair<'_, Rule>) -> Result<ValueDefinition, String> {
        // println!("value definition");
        let mut name: Option<String> = None;
        let mut regex_pattern: Option<String> = None;
        let mut regex_val: Option<Regex> = None;
        let mut options: Option<String> = None;
        let mut is_filldown = false;
        let mut is_key = false;
        let mut is_required = false;
        let mut is_list = false;
        let mut is_fillup = false;

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
        if let (Some(name), Some(mut regex_pattern)) = (name.clone(), regex_pattern.clone()) {
            if let Some(ref opts) = options {
                let opts = opts.split(",");
                for word in opts {
                    match word {
                        "Filldown" => is_filldown = true,
                        "Key" => is_key = true,
                        "Required" => is_required = true,
                        "List" => is_list = true,
                        "Fillup" => is_fillup = true,
                        x => panic!("Unknown option {:?}", &x),
                    }
                }
            }
            if regex_pattern.contains(r#"\<"#) {
                println!("WARNING: replacing \\< with < in value '{}'", &name);
                regex_pattern = regex_pattern.replace("\\<", "<");
            }
            if regex_pattern.contains(r#"\>"#) {
                println!("WARNING: replacing \\> with > in value '{}'", &name);
                regex_pattern = regex_pattern.replace("\\>", ">");
            }
            Ok(ValueDefinition {
                name,
                regex_pattern,
                is_filldown,
                is_key,
                is_required,
                is_list,
                is_fillup,
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
    ) -> Result<(HashMap<String, ValueDefinition>, Vec<String>), String> {
        let mut vals = HashMap::new();
        let mut mandatory_values: Vec<String> = vec![];
        for pair in pair.clone().into_inner() {
            if Rule::value_definition == pair.as_rule() {
                let val = Self::parse_value_definition(&pair)?;
                if val.is_required {
                    mandatory_values.push(val.name.clone());
                }
                vals.insert(val.name.clone(), val);
            }
        }
        Ok((vals, mandatory_values))
    }
    pub fn from_file(fname: &str) -> Self {
        // println!("Path: {}", &fname);
        let template = std::fs::read_to_string(&fname).expect("File read failed");
        // pad with a newline, because dealing with a missing one within grammar is a PITA
        let template = format!("{}\n\n\n", template);

        let mut seen_eoi = false;
        let mut values: HashMap<String, ValueDefinition> = HashMap::new();
        let mut states: HashMap<String, StateCompiled> = HashMap::new();
        let mut mandatory_values: Vec<String> = vec![];

        let end_state = NextState::NamedState(format!("End"));
        let eof_rule = StateRule {
            rule_match: format!(".*"),
            transition: RuleTransition {
                line_action: LineAction::Next(Some(end_state)),
                record_action: RecordAction::Record,
            },
        };

        let compiled_eof_rule = Self::compile_state_rule(&eof_rule, &values).unwrap();

        let eof_state = StateCompiled {
            name: format!("EOF"),
            rules: vec![compiled_eof_rule],
        };
        states.insert(eof_state.name.clone(), eof_state);

        match TextFSMParser::parse(Rule::file, &template) {
            Ok(pairs) => {
                for pair in pairs.clone() {
                    match pair.as_rule() {
                        Rule::value_definitions => {
                            (values, mandatory_values) = Self::parse_value_defs(&pair).unwrap();
                        }
                        Rule::state_definitions => {
                            for pair in pair.clone().into_inner() {
                                match pair.as_rule() {
                                    Rule::state_definition => {
                                        trace!("STATE DEFINITION");
                                        Self::_log_pair(0, &pair);
                                        let state = Self::parse_and_compile_state_definition(
                                            &pair, &values,
                                        )
                                        .unwrap();
                                        trace!("STATE DEFINITION END: {:?}", &state);
                                        if &state.name != "EOF" {
                                            if states.get(&state.name).is_some() {
                                                panic!(
                                                    "State {} already defined in the file!",
                                                    &state.name
                                                );
                                            }
                                        }
                                        states.insert(state.name.clone(), state);
                                    }
                                    x => {
                                        panic!("state definition rule {:?} not supported", x);
                                    }
                                }
                            }
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

                if !seen_eoi {
                    println!("WARNING: EOI token not seen");
                }

                // FIXME: check that the "Start" state exists
                return TextFSMParser {
                    values,
                    mandatory_values,
                    states,
                };
            }
            Err(e) => panic!("file {} Error: {}", &fname, e),
        }
    }
}

impl TextFSM {
    pub fn from_file(fname: &str) -> Self {
        let parser = TextFSMParser::from_file(fname);
        let curr_state = format!("Start");
        TextFSM {
            parser,
            curr_state,
            ..Default::default()
        }
    }

    pub fn set_curr_state(&mut self, state_name: &str) {
        if state_name != "End" {
            if self.parser.states.get(state_name).is_none() {
                panic!("State '{}' not found!", state_name);
            }
        }
        self.curr_state = state_name.to_string();
    }

    pub fn is_filldown_value(&self, value_name: &str) -> Option<bool> {
        if let Some(ref val) = self.parser.values.get(value_name) {
            Some(val.is_filldown)
        } else {
            None
        }
    }

    pub fn is_list_value(&self, value_name: &str) -> Option<bool> {
        if let Some(ref val) = self.parser.values.get(value_name) {
            Some(val.is_list)
        } else {
            None
        }
    }

    pub fn insert_value(
        &self,
        typ: &str,
        curr_record: &mut DataRecord,
        filldown_record: &mut DataRecord,
        name: &str,
        maybe_value: Option<String>,
        aline: &str,
    ) {
        let ins_value = if let Some(value) = maybe_value {
            trace!("{} SET VAR '{}' = '{}'", &typ, &name, &value.as_str());
            if self.is_list_value(name).expect("is list?") {
                Value::List(vec![value.clone()])
            } else {
                Value::Single(value.clone())
            }
        } else {
            error!(
                "WARNING: {} Could not capture '{}' from string '{}'",
                typ, name, aline
            );
            if self.is_list_value(name).expect("is list?") {
                Value::List(vec![format!("None")])
            } else {
                Value::Single(format!(""))
            }
        };
        curr_record.0.insert(name.to_string(), ins_value.clone());
        if self.is_filldown_value(name).unwrap() {
            filldown_record
                .0
                .insert(name.to_string(), ins_value.clone());
        }
    }

    pub fn parse_line(&mut self, aline: &str) -> Option<NextState> {
        let maybe_next_state: Option<NextState> = None;

        let curr_state = self.curr_state.clone();

        if let Some(ref curr_state) = self.parser.states.get(&curr_state) {
            trace!("CURR STATE: {:?}", &curr_state);
            for rule in &curr_state.rules {
                let mut transition: RuleTransition = Default::default();
                transition.line_action = LineAction::Continue;
                trace!("TRY RULE: {:?}", &rule);
                let mut capture_matched = false;
                let mut tmp_datarec = DataRecord::new();
                let mut tmp_filldown_rec = DataRecord::new();
                match &rule.maybe_regex {
                    Some(MultiRegex::Classic(rx)) => {
                        debug!("RULE(CLASSIC REGEX): {:?}", &rule);
                        for caps in rx.captures_iter(aline) {
                            for name in &rule.match_variables {
                                let maybe_value = caps.name(name).map(|x| x.as_str().to_string());
                                self.insert_value(
                                    "CLASSIC",
                                    &mut tmp_datarec,
                                    &mut tmp_filldown_rec,
                                    name,
                                    maybe_value,
                                    aline,
                                );
                            }
                            capture_matched = true;
                        }
                    }
                    Some(MultiRegex::Fancy(rx)) => {
                        debug!("RULE(FANCY REGEX): {:?}", &rule);
                        for caps in rx.captures_iter(aline) {
                            for name in &rule.match_variables {
                                if let Ok(ref caps) = caps {
                                    let maybe_value =
                                        caps.name(name).map(|x| x.as_str().to_string());
                                    self.insert_value(
                                        "FANCY",
                                        &mut tmp_datarec,
                                        &mut tmp_filldown_rec,
                                        name,
                                        maybe_value,
                                        aline,
                                    );
                                } else {
                                    panic!("FANCY caps not ok");
                                }
                            }
                            capture_matched = true;
                        }
                    }
                    x => {
                        panic!("Regex {:?} on rule is not supported", &x);
                    }
                }
                if capture_matched {
                    trace!("TMP_REC: {:?}", &tmp_datarec);
                    trace!("TMP_FILLDOWN: {:?}", &tmp_filldown_rec);
                    for (k, v) in tmp_datarec.0 {
                        self.curr_record.append_value(k, v);
                    }
                    self.filldown_record.overwrite_from(tmp_filldown_rec);
                    transition = rule.transition.clone();
                }
                // println!("TRANS: {:?}", &transition);

                match transition.record_action {
                    RecordAction::Record => {
                        let mut mandatory_count = 0;
                        let number_of_values = self.curr_record.keys().len();

                        for k in &self.parser.mandatory_values {
                            if self.curr_record.get(k).is_some() {
                                mandatory_count += 1;
                            }
                        }
                        if number_of_values > 0 {
                            if mandatory_count == self.parser.mandatory_values.len() {
                                let mut new_rec: DataRecord = Default::default();
                                /* fill the record from filldown */
                                new_rec = self.filldown_record.clone();
                                /* swap with the current record */
                                std::mem::swap(&mut new_rec, &mut self.curr_record);
                                // Set the values that aren't set yet - FIXME: this feature should be
                                // possible to be disabled as "" and nothing are very different things.
                                for (_k, v) in &self.parser.values {
                                    if new_rec.get(&v.name).is_none() {
                                        if self.is_list_value(&v.name).expect("is list?") {
                                            new_rec.0.insert(v.name.clone(), Value::List(vec![]));
                                        } else {
                                            new_rec
                                                .0
                                                .insert(v.name.clone(), Value::Single(format!("")));
                                        }
                                    }
                                }
                                trace!("RECORD: {:?}", &new_rec);
                                self.records.push(new_rec);
                            } else {
                                trace!("RECORD: no required fields set");
                            }
                        } else {
                            trace!("RECORD: record is empty, not dumping");
                        }
                    }
                    RecordAction::NoRecord => {}
                    RecordAction::Clear => {
                        let mut rem_keys: Vec<String> = vec![];
                        for (ref k, ref _v) in self.curr_record.iter() {
                            if !self.is_filldown_value(&k).expect("Variable does not exist") {
                                rem_keys.push(k.to_string());
                            }
                        }
                        for k in rem_keys {
                            self.curr_record.remove(&k);
                        }
                    }
                    RecordAction::Clearall => {
                        // reset the current record
                        self.curr_record = Default::default();
                        self.filldown_record = Default::default();
                    }
                }
                match transition.line_action {
                    LineAction::Next(x) => return x,
                    LineAction::Continue => {}
                }
            }
        } else {
            panic!("State {} not found!", &self.curr_state);
        }
        maybe_next_state
    }

    pub fn lowercase_keys(src: &Vec<DataRecord>) -> Vec<DataRecord> {
        let mut out = vec![];

        for irec in src {
            let mut hm = DataRecord::new();
            for (k, v) in irec.iter() {
                let kl = k.to_lowercase();
                hm.0.insert(kl, v.clone());
            }
            out.push(hm);
        }
        out
    }

    pub fn parse_file(
        &mut self,
        fname: &str,
        conversion: Option<DataRecordConversion>,
    ) -> Vec<DataRecord> {
        let input = std::fs::read_to_string(&fname).expect("Data file read failed");
        for (lineno, aline) in input.lines().enumerate() {
            debug!("LINE:#{}:'{}'", lineno + 1, &aline);
            if let Some(next_state) = self.parse_line(&aline) {
                match next_state {
                    NextState::Error(maybe_msg) => {
                        panic!("Error state reached! msg: {:?}", &maybe_msg);
                    }
                    NextState::NamedState(name) => {
                        self.set_curr_state(&name);
                    }
                }
            }
            if &self.curr_state == "EOF" || &self.curr_state == "End" {
                break;
            }
        }
        if &self.curr_state != "End" {
            self.set_curr_state("EOF");
            self.parse_line("");
            // FIXME: Can EOF state transition into something else ? Presumably not.
            self.set_curr_state("End");
        }
        match conversion {
            None => self.records.clone(),
            Some(DataRecordConversion::LowercaseKeys) => Self::lowercase_keys(&self.records),
        }
    }
}
