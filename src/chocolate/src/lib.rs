use std::{fmt, io};

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
	#[error("unexpected char `{ch}` at row {row}, col {col}")]
	Syntax { ch: char, row: u32, col: u32 },

	#[error("program not finished in {limit} steps")]
	Timeout { limit: u32 },

	#[error("r command costs exceeded limit of {limit}")]
	TooMuchRotate { limit: u32 },

	#[error("rows * cols > {limit}")]
	CodeIsBig { limit: u32 },

	#[error("top left is blank")]
	BlankStart,

	#[error("non-ascii char `{ch}`")]
	NonAscii { ch: char },

	#[error("overflow `{ch}` at row {row}, col {col}")]
	Overflow { ch: char, row: u32, col: u32 },
}

fn getc(input: &mut impl io::BufRead) -> io::Result<Option<u8>> {
	let value = input.fill_buf()?.first().copied();
	input.consume(1);
	Ok(value)
}

fn putc(out: &mut impl io::Write, byte: u8) -> io::Result<()> {
	out.write_all(&[byte])
}

type Matrix<T> = Box<[Box<[T]>]>;

fn parse_raw_code(source: &str, code_limit: u32) -> Result<Matrix<u8>, Error> {
	let mut max_cols = 0usize;
	let mut code = Vec::new();

	for line in source.split('\n') {
		let mut row = Vec::new();
		for ch in line.trim_end().chars() {
			if ch.is_ascii() {
				row.push(ch as u8);
			} else {
				return Err(Error::NonAscii { ch });
			}
		}

		if row.is_empty() {
			break;
		}

		max_cols = max_cols.max(row.len());
		code.push(row.into_boxed_slice())
	}

	let size = max_cols.saturating_mul(code.len());
	if size > code_limit as usize {
		return Err(Error::CodeIsBig { limit: code_limit });
	}

	let first = code.first().and_then(|x| x.first());
	if first.copied().unwrap_or(b' ') == b' ' {
		return Err(Error::BlankStart);
	}

	Ok(code.into_boxed_slice())
}

type Coord = [u32; 2];

fn to_signed_coord([r, c]: Coord) -> [i64; 2] {
	[r as i64, c as i64]
}

#[derive(Default, Clone)]
pub struct Block {
	pub cmd: u8,
	pub len: usize,
	pub fronts: [[Option<Coord>; 2]; 4],
}

impl fmt::Debug for Block {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Block")
			.field("cmd", &(self.cmd as char))
			.field("len", &self.len)
			.field("fronts", &self.fronts)
			.finish()
	}
}

fn get_matrix<T: Copy>(
	matrix: &Matrix<T>,
	row: u32,
	col: u32,
	default: T,
) -> T {
	matrix
		.get(row as usize)
		.and_then(|x| x.get(col as usize))
		.copied()
		.unwrap_or(default)
}

fn set_matrix<T>(matrix: &mut Matrix<T>, row: u32, col: u32, value: T) {
	if let Some(x) = matrix
		.get_mut(row as usize)
		.and_then(|x| x.get_mut(col as usize))
	{
		*x = value;
	}
}

#[derive(Debug)]
pub struct Code {
	pub area_indices: Matrix<u32>,
	pub blocks: Box<[Block]>,
}

fn parse(raw_code: &Matrix<u8>) -> Code {
	let mut area_indices = Vec::with_capacity(raw_code.len());
	for row in raw_code {
		let row = vec![0u32; row.len()];
		area_indices.push(row.into_boxed_slice());
	}
	let mut area_indices = area_indices.into_boxed_slice();
	let mut blocks = Vec::new();
	let mut block_index = 1;

	for (row, r) in raw_code.iter().zip(0..) {
		for (&cmd, c) in row.iter().zip(0..) {
			if cmd == b' ' || get_matrix(&area_indices, r, c, 0) != 0 {
				continue;
			}

			let mut block = Block {
				cmd,
				..Default::default()
			};
			for dp in block.fronts.iter_mut() {
				for cc in dp {
					*cc = Some([r, c]);
				}
			}

			let mut stack = vec![[r, c]];
			while let Some([r, c]) = stack.pop() {
				set_matrix(&mut area_indices, r, c, block_index);
				block.len += 1;

				for dp in 0..4 {
					for cc in 0..2 {
						let f = &mut block.fronts[dp][cc];
						let is_in_boundary = {
							let [fr, fc] = f.map_or([-1, -1], to_signed_coord);
							let [r, c] = to_signed_coord([r, c]);
							[
								[[c, fr] > [fc, r], [c, r] > [fc, fr]],
								[[r, c] > [fr, fc], [r, fc] > [fr, c]],
								[[fc, r] > [c, fr], [fc, fr] > [c, r]],
								[[fr, fc] > [r, c], [fr, c] > [r, fc]],
							][dp][cc]
						};
						if is_in_boundary {
							*f = Some([r, c]);
						}
					}
				}

				let mut next_coords = Vec::with_capacity(4);
				if r > 0 {
					next_coords.push([r - 1, c]);
				}
				next_coords.push([r + 1, c]);
				if c > 0 {
					next_coords.push([r, c - 1]);
				}
				next_coords.push([r, c + 1]);

				for [r, c] in next_coords {
					if get_matrix(raw_code, r, c, b' ') == cmd
						&& get_matrix(&area_indices, r, c, u32::MAX) == 0
					{
						set_matrix(&mut area_indices, r, c, block_index);
						stack.push([r, c])
					}
				}
			}

			let step_r = [0, 1, 0, -1];
			let step_c = [1, 0, -1, 0];
			for dp in 0..4 {
				let step_r = step_r[dp];
				let step_c = step_c[dp];
				for cc in 0..2 {
					let front = &mut block.fronts[dp][cc];
					let [front_r, front_c] =
						front.map_or([-1, -1], to_signed_coord);
					let next_r = front_r + step_r;
					let next_c = front_c + step_c;
					if 0 <= next_r
						&& 0 <= next_c && get_matrix(
						raw_code,
						next_r as u32,
						next_c as u32,
						b' ',
					) != b' '
					{
						*front = Some([next_r as u32, next_c as u32]);
					} else {
						*front = None;
					}
				}
			}
			blocks.push(block);
			block_index += 1;
		}
	}

	Code {
		area_indices,
		blocks: blocks.into_boxed_slice(),
	}
}

pub struct RunState<I, O> {
	pub input: I,
	pub output: O,
	pub stack: Vec<i64>,
	pub code: Code,
	row: u32,
	col: u32,
	pub dp: u8,
	pub cc: u8,
	pub timeout_limit: u32,
	pub steps: u32,
	pub r_limit: u32,
	pub r_sum: u32,
	pub finished: bool,
}

impl<I: io::BufRead, O: io::Write> RunState<I, O> {
	fn new(
		code: Code,
		input: I,
		output: O,
		timeout_limit: u32,
		r_limit: u32,
	) -> Self {
		Self {
			input,
			output,
			stack: Vec::new(),
			code,
			row: 0,
			col: 0,
			dp: 0,
			cc: 0,
			timeout_limit,
			steps: 0,
			r_limit,
			r_sum: 0,
			finished: false,
		}
	}

	pub fn from_source(
		source: &str,
		input: I,
		output: O,
		code_limit: u32,
		timeout_limit: u32,
		r_limit: u32,
	) -> Result<Self, Error> {
		let code = parse(&parse_raw_code(source, code_limit)?);
		Ok(RunState::new(code, input, output, timeout_limit, r_limit))
	}

	fn cmd_input(&mut self) {
		if let Ok(Some(ch)) = getc(&mut self.input) {
			self.stack.push(ch as i64)
		}
	}

	fn cmd_output(&mut self) {
		if let Some(x) = self.stack.pop() {
			let _ = putc(&mut self.output, (x & 255) as u8);
		}
	}

	#[inline]
	fn overflow_error(&self, ch: char) -> Error {
		Error::Overflow {
			ch,
			row: self.row,
			col: self.col,
		}
	}

	fn cmd_push(&mut self, len: usize) -> Result<(), Error> {
		let Ok(len) = i64::try_from(len) else {
			return Err(self.overflow_error('P'));
		};
		self.stack.push(len);
		Ok(())
	}

	fn cmd_pop(&mut self) {
		self.stack.pop();
	}

	fn two(
		&mut self,
		op: char,
		func: fn(i64, i64) -> Option<i64>,
	) -> Result<(), Error> {
		let Some(&[y, x]) = self.stack.last_chunk() else {
			return Ok(());
		};
		self.stack.pop();
		self.stack.pop();
		if let Some(z) = func(y, x) {
			self.stack.push(z);
			Ok(())
		} else {
			Err(self.overflow_error(op))
		}
	}

	fn cmd_add(&mut self) -> Result<(), Error> {
		self.two('+', i64::checked_add)
	}

	fn cmd_sub(&mut self) -> Result<(), Error> {
		self.two('+', i64::checked_sub)
	}

	fn cmd_mul(&mut self) -> Result<(), Error> {
		self.two('+', i64::checked_mul)
	}

	fn cmd_div(&mut self) -> Result<(), Error> {
		let Some(&[y, x]) = self.stack.last_chunk() else {
			return Ok(());
		};
		if x == 0 {
			return Ok(());
		}
		self.stack.pop();
		self.stack.pop();
		let mut div = y / x;
		let rem = y % x;
		if rem != 0 && ((y >= 0) != (x >= 0)) {
			div -= 1;
		}
		self.stack.push(div);
		Ok(())
	}

	fn cmd_rem(&mut self) -> Result<(), Error> {
		let Some(&[y, x]) = self.stack.last_chunk() else {
			return Ok(());
		};
		if x == 0 {
			return Ok(());
		}
		self.stack.pop();
		self.stack.pop();
		let mut rem = y % x;
		if rem != 0 && ((y >= 0) != (x >= 0)) {
			rem += x;
		}
		self.stack.push(rem);
		Ok(())
	}

	fn cmd_gt(&mut self) {
		let Some(&[y, x]) = self.stack.last_chunk() else {
			return;
		};
		self.stack.pop();
		self.stack.pop();
		self.stack.push(if y > x { 1 } else { 0 });
	}

	fn cmd_not(&mut self) {
		if let Some(x) = self.stack.pop() {
			self.stack.push(if x == 0 { 1 } else { 0 });
		}
	}

	fn cmd_dp(&mut self) {
		if let Some(x) = self.stack.pop() {
			self.dp = (self.dp + (x & 3) as u8) & 3;
		}
	}

	fn cmd_cc(&mut self) {
		if let Some(x) = self.stack.pop() {
			self.cc = (self.cc + (x & 1) as u8) & 1;
		}
	}

	fn cmd_dup(&mut self) {
		if let Some(&x) = self.stack.last() {
			self.stack.push(x);
		}
	}

	fn cmd_rotate(&mut self) -> Result<(), Error> {
		let Some(&[y, x]) = self.stack.last_chunk() else {
			return Ok(());
		};
		if y <= 0 || y as usize + 2 > self.stack.len() {
			return Ok(());
		}
		self.stack.pop();
		self.stack.pop();

		self.r_sum = self
			.r_sum
			.saturating_add(u32::try_from(y).unwrap_or(u32::MAX));

		let x = x.rem_euclid(y);
		if x == 0 {
			return Ok(());
		}

		let x = y - x;

		let base = self.stack.len() - y as usize;
		let mut rolled = Vec::new();
		for i in (x as usize)..(y as usize) {
			rolled.push(self.stack[i + base]);
		}
		for i in 0..(x as usize) {
			rolled.push(self.stack[i + base]);
		}
		self.stack[base..(base + y as usize)]
			.clone_from_slice(&rolled[0..(y as usize)]);

		if self.r_sum > self.r_limit {
			Err(Error::TooMuchRotate {
				limit: self.r_limit,
			})
		} else {
			Ok(())
		}
	}

	pub fn next_block_index(&self) -> u32 {
		get_matrix(&self.code.area_indices, self.row, self.col, 0)
	}

	pub fn next_block(&self) -> Option<&Block> {
		let index = self.next_block_index() as usize;
		if index == 0 {
			return None;
		}
		self.code.blocks.get(index - 1)
	}

	fn step_inner(&mut self) -> Result<(), Error> {
		if self.steps > self.timeout_limit {
			return Err(Error::Timeout {
				limit: self.timeout_limit,
			});
		}

		let Some(block) = self.next_block().cloned() else {
			// 일어날 수 없긴 한데 에러는 뱉어보자
			return Err(Error::Syntax {
				ch: '뷁',
				row: self.row,
				col: self.col,
			});
		};

		match block.cmd {
			b'I' => self.cmd_input(),
			b'O' => self.cmd_output(),
			b'P' => self.cmd_push(block.len)?,
			b'p' => self.cmd_pop(),
			b'+' => self.cmd_add()?,
			b'-' => self.cmd_sub()?,
			b'*' => self.cmd_mul()?,
			b'/' => self.cmd_div()?,
			b'%' => self.cmd_rem()?,
			b'!' => self.cmd_not(),
			b'>' => self.cmd_gt(),
			b'D' => self.cmd_dp(),
			b'C' => self.cmd_cc(),
			b'd' => self.cmd_dup(),
			b'r' => self.cmd_rotate()?,
			c => {
				return Err(Error::Syntax {
					ch: c as char,
					row: self.row,
					col: self.col,
				});
			}
		}

		let mut next = None;
		for i in 0..8 {
			let front = block.fronts[self.dp as usize][self.cc as usize];
			if front.is_some() {
				next = front;
				break;
			}
			if i % 2 == 1 {
				self.dp = (self.dp + 1) & 3;
			} else {
				self.cc = (self.cc + 1) & 1;
			}
		}

		if let Some([r, c]) = next {
			self.row = r;
			self.col = c;
			self.steps += 1;
		} else {
			self.finished = true;
		}

		Ok(())
	}

	pub fn step(&mut self) -> Result<(), Error> {
		let res = self.step_inner();
		if res.is_err() {
			self.finished = true;
		}
		res
	}
}

pub fn run(
	source: &str,
	input: &mut impl io::BufRead,
	output: &mut impl io::Write,
) -> Result<(), Error> {
	let mut state = RunState::from_source(
		source, input, output, 1_000_000, 1_000_000, 1_000_000,
	)?;
	while !state.finished {
		state.step()?;
	}
	Ok(())
}
