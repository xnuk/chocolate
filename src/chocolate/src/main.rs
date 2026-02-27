use std::{env, fs, io};

use chocolate::run;

fn main() {
	let filepath = env::args_os().nth(1).expect("Usage: chocolate [FILE]");
	let content = fs::read_to_string(filepath).unwrap();
	run(&content, &mut io::stdin().lock(), &mut io::stdout().lock()).unwrap();
}
