// pub mod bitpack;
// pub mod memory;
// pub mod machine;
use std::env;
use rum::machine;

fn main() {
	let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let instructions = machine::boot(filename);
    machine::run(instructions);
}
