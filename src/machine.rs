use std::fs::File;
use byteorder::{ReadBytesExt, BigEndian};
use std::io::prelude::*;
use std::io::{stdout, stdin, Cursor};
use std::process;
use std::convert::TryInto;

use crate::memory;
use crate::bitpack;

// pub mod machine {
	pub fn run(program: Vec<u32>) -> () {
		let mut segmap = memory::Memory::new(program);
		// next, start calling decode() on each instruction
		// and dispatch it!
		let mut r = [0_u32; 8];
		let mut pc = 0_usize;
		loop {
			let instruction = Instruction::decode(segmap.get_instruction(pc));
			match instruction {
				Some(instr) => {
					let op = instr.opcode;
					pc += 1;
					match op {
						Opcode::CMov => {
							let ra = instr.ra.unwrap();
							let rb = instr.rb.unwrap();
							let rc = instr.rc.unwrap();
							if r[rc] != 0 { r[ra] = r[rb] };
						},
						Opcode::Load => {
							let ra = instr.ra.unwrap();
							let rb = instr.rb.unwrap();
							let rc = instr.rc.unwrap();
							r[ra] = segmap.load(r[rb].try_into().unwrap(), r[rc].try_into().unwrap()).unwrap();
						},
						Opcode::Store => {
							let ra = instr.ra.unwrap();
							let rb = instr.rb.unwrap();
							let rc = instr.rc.unwrap();
							segmap.store(r[ra].try_into().unwrap(), r[rb].try_into().unwrap(), r[rc].try_into().unwrap());
						},
						Opcode::Add => {
							let ra = instr.ra.unwrap();
							let rb = instr.rb.unwrap();
							let rc = instr.rc.unwrap();
							r[ra] = r[rb] + r[rc];
						},
						Opcode::Mul => {
							let ra = instr.ra.unwrap();
							let rb = instr.rb.unwrap();
							let rc = instr.rc.unwrap();
							r[ra] = r[rb] * r[rc];
						},
						Opcode::Div => {
							let ra = instr.ra.unwrap();
							let rb = instr.rb.unwrap();
							let rc = instr.rc.unwrap();
							r[ra] = r[rb] / r[rc];
						},
						Opcode::Nand => {
							let ra = instr.ra.unwrap();
							let rb = instr.rb.unwrap();
							let rc = instr.rc.unwrap();
							r[ra] = !(r[rb] & r[rc]);
						},
						Opcode::Halt => {
							process::exit(0);
						},
						Opcode::MapSegment => {
							let rb = instr.rb.unwrap();
							let rc = instr.rc.unwrap();
							r[rb] = segmap.allocate(r[rc].try_into().unwrap()).try_into().unwrap();
						},
						Opcode::UnmapSegment => {
							let rc = instr.rc.unwrap();
							segmap.deallocate(r[rc].try_into().unwrap());
						},
						Opcode::Output => {
							let rc = instr.rc.unwrap();
							let value = r[rc] as u8;
							stdout().write(&[value]).unwrap();
						},
						Opcode::Input => {
							let rc = instr.rc.unwrap();
							match stdin().bytes().next().unwrap() {
								Ok(value) => {
									r[rc] = value as u32;
								},
								Err(e) => panic!("Bad input: {}", e)
							}
						},
						Opcode::LoadProgram => {
							let rb = instr.rb.unwrap();
							let rc = instr.rc.unwrap();
							if r[rb] != 0 {
								segmap.load_segment(r[rb].try_into().unwrap());
							}
							pc = r[rc].try_into().unwrap();
						},
						Opcode::LoadValue => {
							let ra = instr.ra.unwrap();
							let value = instr.value.unwrap();
							r[ra] = value;
						},
						Opcode::Err => {
							panic!("Illegal instruction!")
						}
					}

				},
				None => panic!("Illegal instruction")
			}


		}

	}

	#[derive(Debug, PartialEq)]
	enum Opcode {
	    CMov,
	    Load,
	    Store,
	    Add,
	    Mul,
	    Div,
	    Nand,
	    Halt,
	    MapSegment,
	    UnmapSegment,
	    Output,
	    Input,
	    LoadProgram,
	    LoadValue,
	    Err // opcode cannot be parsed (14 or 15)
	}

	pub fn boot(filename: &str) -> Vec<u32> {
	    // let mut f = File::open(filename).expect("File not found");
	    let mut f = File::open(filename).unwrap_or_else(|_| panic!("File not found: {}", filename));
	    let mut contents = Vec::new();
	    let mut program = Vec::new();

	    match f.read_to_end(&mut contents) {
	        Ok(bytes) => {
	            println!("read {} bytes from {}", bytes, filename);
	            // thanks to jgrillo for the following
	            for i in 0..contents.len() / 4 {
	                let idx = i * 4;
	                let buf = &contents[idx..idx + 4];
	                let mut rdr = Cursor::new(buf);
	                program.push(rdr.read_u32::<BigEndian>().unwrap());
	            }

	            program
	        },
	        Err(e) => panic!("Encountered error while reading from {}: {}", filename, e)
	    }
	}

	// functions for instruction parsing.

	fn parse_opcode(instruction: u32) -> Opcode {
	    let opcode = bitpack::bitpack::getu(instruction as u64, 4, 28);

	    match opcode {
	        0 => Opcode::CMov,
	        1 => Opcode::Load,
	        2 => Opcode::Store,
	        3 => Opcode::Add,
	        4 => Opcode::Mul,
	        5 => Opcode::Div,
	        6 => Opcode::Nand,
	        7 => Opcode::Halt,
	        8 => Opcode::MapSegment,
	        9 => Opcode::UnmapSegment,
	        10 => Opcode::Output,
	        11 => Opcode::Input,
	        12 => Opcode::LoadProgram,
	        13 => Opcode::LoadValue,
	        _ => Opcode::Err
	    }
	}

	#[derive(Debug)]
	struct Instruction {
	    opcode: Opcode,
	    ra: Option<usize>,
	    rb: Option<usize>,
	    rc: Option<usize>,
	    value: Option<u32>
	}

	impl Instruction {
		fn decode(instruction: u32) -> Option<Instruction> {
			let opcode = parse_opcode(instruction);
			let ra = match opcode {
				// Opcode::LoadValue => Some(bitpack::bitpack::getu(instruction as u64, 3, 25) as usize),
				Opcode::LoadValue => Some((((instruction) >> 25) & 0x7) as usize),
				Opcode::Err => None,
				// _ => Some(bitpack::bitpack::getu(instruction as u64, 3, 6) as usize)
				_ => Some((((instruction) >> 6) & 0x7) as usize)
			};
			let rb = match opcode {
				Opcode::LoadValue | Opcode::Err => None,
				// _ => Some(bitpack::bitpack::getu(instruction as u64, 3, 3) as usize)
				_ => Some((((instruction) >> 3) & 0x7) as usize)
			};
			let rc = match opcode {
				Opcode::LoadValue | Opcode::Err => None,
				// _ => Some(bitpack::bitpack::getu(instruction as u64, 3, 0) as usize)
				_ => Some((instruction & 0x7) as usize)
			};
			let value = match opcode {
				// Opcode::LoadValue => Some(bitpack::bitpack::getu(instruction as u64, 25, 0) as u32),
				Opcode::LoadValue => Some(((instruction << 7) >> 7) as u32),
				_ => None
			};
			Some(Instruction {
				opcode, ra, rb, rc, value
			})
		}
	}
// }