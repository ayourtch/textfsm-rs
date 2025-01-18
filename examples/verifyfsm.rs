use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use textfsm_rs::*;

#[derive(Serialize, Deserialize)]
struct ParsedSample {
    parsed_sample: Vec<HashMap<String, String>>,
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

            let mut only_in_yaml: Vec<Vec<String>> = vec![];
            let mut only_in_parse: Vec<Vec<String>> = vec![];

            for (i, irec) in result.iter().enumerate() {
                let mut vo: Vec<String> = vec![];
                for (k, _v) in irec {
                    if i >= yaml_map.parsed_sample.len()
                        || yaml_map.parsed_sample[i].get(k).is_none()
                    {
                        vo.push(k.clone());
                    }
                }
                only_in_parse.push(vo);
            }

            for (i, irec) in yaml_map.parsed_sample.iter().enumerate() {
                let mut vo: Vec<String> = vec![];
                for (k, _v) in irec {
                    if result[i].get(k).is_none() {
                        vo.push(k.clone());
                    }
                }
                only_in_yaml.push(vo);
            }

            println!("Only in yaml: {:?}", &only_in_yaml);
            println!("Only in parse: {:?}", &only_in_parse);
        }
    } else {
        println!("Could not load YAML!");
    }
}
