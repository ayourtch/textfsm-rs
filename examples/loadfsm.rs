use textfsm_rs::*;

fn main() {
    for arg in std::env::args().skip(1) {
        // println!("Reading file {}", &arg);
        let textfsm = TextFSM::from_file(&arg);
        println!("FSM: {:#?}", &textfsm);
    }
}
