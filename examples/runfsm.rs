use textfsm_rs::*;

fn main() {
    let template_name = std::env::args()
        .nth(1)
        .expect("Missing TextFSM template file name");
    let data_name = std::env::args()
        .nth(2)
        .expect("Missing TextFSM data file name");
    let mut textfsm = TextFSM::from_file(&template_name);
    let result = textfsm.parse_file(&data_name);
    println!("Records: {:?}", &result);
}
