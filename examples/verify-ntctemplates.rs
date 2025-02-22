use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use textfsm_rs::*;

#[derive(Serialize, Deserialize)]
struct ParsedSample {
    parsed_sample: Vec<DataRecord>,
}

enum VerifyResult {
    CouldNotLoadYaml,
    VerifySuccess,
    ResultsDiffer,
}

fn verify(template_name: &str, data_name: &str, yaml_verify_name: &str) -> VerifyResult {
    let mut textfsm = TextFSM::from_file(&template_name);
    let yaml = std::fs::read_to_string(&yaml_verify_name).expect("YAML File read failed");

    let result = textfsm.parse_file(&data_name, Some(DataRecordConversion::LowercaseKeys));
    println!("RESULT: {:?}\n", &result);
    if let Ok(yaml_map) = serde_yaml::from_str::<ParsedSample>(&yaml) {
        if result == yaml_map.parsed_sample {
            println!("Parsed result matches YAML");
            VerifyResult::VerifySuccess
        } else {
            println!("Results differ");
            println!("Records: {:?}", &result);
            println!("\n");
            println!("yaml: {:?}", &yaml_map.parsed_sample);
            println!("\n");

            let (only_in_parse, only_in_yaml) =
                DataRecord::compare_sets(&result, &yaml_map.parsed_sample);

            let mut mismatch_count = 0;
            for x in &only_in_yaml {
                mismatch_count += x.len();
            }
            for x in &only_in_parse {
                mismatch_count += x.len();
            }
            println!("Only in yaml: {:?}", &only_in_yaml);
            println!("Only in parse: {:?}", &only_in_parse);
            println!("\n\n");
            if mismatch_count == 0 {
                println!("Results differ, but only by order");
                VerifyResult::VerifySuccess
            } else {
                println!("Results differ");
                VerifyResult::ResultsDiffer
            }
        }
    } else {
        println!("WARNING: YAML did not load correctly!");
        panic!("Could not load YAML");
        VerifyResult::CouldNotLoadYaml
    }
}

fn collect_file_names(template_dir: &str, extension: &str) -> Result<Vec<String>, std::io::Error> {
    let mut base_names = Vec::new();

    for entry in std::fs::read_dir(template_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some(extension) {
            if let Some(base_name) = path.file_stem().and_then(|name| name.to_str()) {
                base_names.push(base_name.to_string());
            }
        }
    }

    Ok(base_names)
}

fn collect_bare_directories(base_dir: &str) -> Result<Vec<String>, std::io::Error> {
    let mut dir_names = Vec::new();

    for entry in std::fs::read_dir(base_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() && !path.extension().is_some() {
            // No extension
            if let Some(dir_name) = path.file_name().and_then(|name| name.to_str()) {
                dir_names.push(dir_name.to_string());
            }
        }
    }

    Ok(dir_names)
}

struct TestRecord {
    template_name: String,
    test_family_name: String,
    test_set_name: String,
    test_data_file_name: String,
    test_yaml_file_name: String,
}

fn main() {
    let root_path = std::env::args()
        .nth(1)
        .expect("missing path to a https://github.com/networktocode/ntc-templates checkout");

    let template_dir = format!("{}/ntc_templates/templates/", &root_path);
    let tests_dir = format!("{}/tests/", &root_path);
    let template_names = collect_file_names(&template_dir, "textfsm")
        .expect("Could not scan the template directory");
    let mut template_names_set = std::collections::HashSet::new();
    for t in &template_names {
        template_names_set.insert(t.clone());
    }
    let test_family_names =
        collect_bare_directories(&tests_dir).expect("Could not scan tests directory");
    println!("{} template names found", template_names.len());
    println!("{} test families found", test_family_names.len());

    let mut all_tests: Vec<TestRecord> = vec![];

    let mut verify_count = 0;
    let mut result_no_yaml_count = 0;
    let mut result_success_count = 0;
    let mut result_differ_count = 0;

    for test_family in &test_family_names {
        let test_family_dir = format!("{}/tests/{}/", &root_path, test_family);
        let test_set_names = collect_bare_directories(&test_family_dir).expect(&format!(
            "Could not scan test family dir {}",
            &test_family_dir
        ));
        for test_set in &test_set_names {
            let candidate_template_name = format!("{}_{}", test_family, test_set);
            if template_names_set.contains(&candidate_template_name) {
                let test_dir = format!("{}/tests/{}/{}/", &root_path, test_family, test_set);
                let test_names = collect_file_names(&test_dir, "raw")
                    .expect("Could not scan the template directory");

                let template_file = format!(
                    "{}/ntc_templates/templates/{}.textfsm",
                    &root_path, &candidate_template_name
                );
                for test_name in &test_names {
                    let data_file = format!(
                        "{}/tests/{}/{}/{}.raw",
                        &root_path, test_family, test_set, test_name
                    );
                    let yaml_file = format!(
                        "{}/tests/{}/{}/{}.yml",
                        &root_path, test_family, test_set, test_name
                    );
                    if std::path::Path::new(&yaml_file).exists() {
                        println!("VERIFY: {} {} {}", &template_file, &data_file, &yaml_file);
                        verify_count += 1;
                        match verify(&template_file, &data_file, &yaml_file) {
                            VerifyResult::CouldNotLoadYaml => {
                                result_no_yaml_count += 1;
                            }
                            VerifyResult::VerifySuccess => {
                                result_success_count += 1;
                            }
                            VerifyResult::ResultsDiffer => {
                                result_differ_count += 1;
                            }
                        }
                    } else {
                        println!("WARNING: raw file exists {} but no yaml", &data_file);
                    }
                }
            } else {
                println!(
                    "WARNING: can not find template for family {} test set {}",
                    test_family, test_set
                );
            }
        }
    }

    println!("\nNTC-TEMPLATES VERIFY RESULTS:");
    println!("   Total tests run: {}", verify_count);
    println!("      Could not load YAML: {}", result_no_yaml_count);
    println!("      Verify success: {}", result_success_count);
    println!("      Results differ: {}", result_differ_count);
}
