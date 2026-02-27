use std::io;

use chocolate::RunState;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct State(RunState<io::Cursor<Vec<u8>>, Vec<u8>>);

macro_rules! transparent {
	($name:ty, $($var:ident = $t:ty),+ $(,)?) => {
		#[wasm_bindgen]
		impl $name {$(
			#[wasm_bindgen(getter)]
			pub fn $var(&self) -> $t { self.0.$var }
		)+}
	};
}

transparent!(
	State,
	dp = u8,
	cc = u8,
	timeout_limit = u32,
	steps = u32,
	r_limit = u32,
	r_sum = u32,
	finished = bool,
);

#[wasm_bindgen]
impl State {
	/// [len, item, item, ..., len, item, item, ...]
	pub fn area_indices_flatten(&self) -> Box<[u32]> {
		let mut ret = Vec::new();
		for row in &self.0.code.area_indices {
			ret.push(row.len() as u32);
			for &x in row {
				ret.push(x);
			}
		}
		ret.into_boxed_slice()
	}

	pub fn stack(&self) -> Box<[i64]> {
		self.0.stack.clone().into_boxed_slice()
	}

	pub fn next_index(&self) -> u32 {
		self.0.next_block_index()
	}

	pub fn next_cmd(&self) -> u8 {
		self.0.next_block().map_or(0, |v| v.cmd)
	}

	pub fn next_len(&self) -> usize {
		self.0.next_block().map_or(0, |v| v.len)
	}

	pub fn output(&self) -> Box<[u8]> {
		self.0.output.clone().into_boxed_slice()
	}

	/// returns empty array if successed, or returns error message if failed.
	pub fn step(&mut self) -> Box<[u8]> {
		match self.0.step() {
			Ok(()) => Box::new([]),
			Err(err) => err.to_string().into_bytes().into_boxed_slice(),
		}
	}

	/// `source` and `input`: must be valid utf-8 string
	/// `error`: where to store utf-8 error message. max length would be 256.
	pub fn from_source_and_input(
		source: &[u8],
		input: &[u8],
		error: &mut [u8],
	) -> Option<State> {
		let source = String::from_utf8_lossy(source).into_owned();
		let state = RunState::from_source(
			&source,
			io::Cursor::new(input.to_vec()),
			Vec::new(),
			1_000_000,
			1_000_000,
			1_000_000,
		);
		match state {
			Ok(x) => Some(State(x)),
			Err(x) => {
				let st = x.to_string();
				let bytes = st.as_bytes();
				let len = error.len().min(bytes.len());
				error[..len].copy_from_slice(&bytes[..len]);
				None
			}
		}
	}
}
