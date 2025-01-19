use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use textfsm_rs::*;

#[derive(Serialize, Deserialize)]
struct ParsedSample {
    parsed_sample: Vec<DataRecord>,
}

fn main() {
    env_logger::init();

    let template_name = std::env::args()
        .nth(1)
        .expect("Missing TextFSM template file name");
    let data_name = std::env::args()
        .nth(2)
        .expect("Missing TextFSM data file name");
    let yaml_verify_name = std::env::args()
        .nth(3)
        .expect("Missing TextFSM verify data YAML file name");
    let mut textfsm = TextFSM::from_file(&template_name);
    let yaml = std::fs::read_to_string(&yaml_verify_name).expect("YAML File read failed");
    let result = textfsm.parse_file(&data_name, Some(DataRecordConversion::LowercaseKeys));
    println!("RESULT: {:?}\n", &result);

    if let Ok(yaml_map) = serde_yaml::from_str::<ParsedSample>(&yaml) {
        if result == yaml_map.parsed_sample {
            println!("Parsed result matches YAML");
        } else {
            println!("Results differ");
            println!("Records: {:?}", &result);
            println!("\n");
            println!("yaml: {:?}", &yaml_map.parsed_sample);
            println!("\n");

            let (only_in_parse, only_in_yaml) =
                DataRecord::compare_sets(&result, &yaml_map.parsed_sample);

            println!("Only in yaml: {:?}", &only_in_yaml);
            println!("Only in parse: {:?}", &only_in_parse);
        }
    } else {
        println!("Could not load YAML!");
    }
}
