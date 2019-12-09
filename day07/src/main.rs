#[macro_use] extern crate itertools;
use itertools::Itertools;

use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

const LOGLEVEL: u8 = 0;
macro_rules! debug {
    ( $($arg:tt)* ) => {
        if LOGLEVEL >= 2 {
            print!("DEBUG: ");
            print!($($arg)*);
        }
    }
}

macro_rules! info {
    ( $($arg:tt)* ) => {
        if LOGLEVEL == 1 {
            print!("INFO: ");
            print!($($arg)*);
        }
    }
}

type Imm = isize;
type Pos = usize;

#[derive(Debug, Clone)]
/// Program struct containing the current state of the emulator
struct Program {
    /// Instruction Pointer
    ip: usize,

    /// Current memory in the emulator
    memory: Vec<isize>,

    /// Lifted instructions to be executed in the emulator
    /// HashMap is keyed by IP of the instruction
    instructions: HashMap<usize, Opcode>,

    /// Input buffer
    input: Vec<isize>,

    /// Output buffer
    output: Vec<isize>,

    /// VM has halted
    halted: bool
}

/// Available opcodes in our computer emulator
/// 
/// Each opcode is appended by how the parameters should be interpretted
///
/// Example:
/// AddAAA - add where all parameters are positions in memory 
/// AddIIA - add where the two parameters are immediates and the result is a position
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Opcode {
    /// add [p1], [p2], [dest]
    AddAAA(Pos, Pos, Pos),

    /// add [p1], imm2, [dest]
    AddAIA(Pos, Imm, Pos),

    /// add imm1, [p1], [dest]
    AddIAA(Imm, Pos, Pos),

    /// add imm1, imm2, [dest]
    AddIIA(Imm, Imm, Pos),

    /// mul [p1], [p2], [dest]
    MulAAA(Pos, Pos, Pos),

    /// mul [p1], imm2, [dest]
    MulAIA(Pos, Imm, Pos),

    /// mul imm1, [p2], [dest]
    MulIAA(Imm, Pos, Pos),

    /// mul imm1, mm2, [dest]
    MulIIA(Imm, Imm, Pos),

    /// input [dest]
    InA(Pos),

    /// output [dest]
    OutA(Pos),

    /// output imm1
    OutI(Imm),

    /// jmpnz [p1], imm2
    /// Reach the value at address p1. If non-zero, jump to imm2
    JumpNonZeroAI(Pos, Imm),

    /// jmpnz imm1, imm2
    /// If p1 is non-zero, jump to imm2
    JumpNonZeroII(Imm, Imm),

    /// jmpnz imm1, [p2]
    /// If p1 is non-zero, read value at address imm2. Jump to the read value.
    JumpNonZeroIA(Imm, Pos),

    /// jmpnz [p1], [p2]
    /// If p1 is non-zero, read value at address imm2. Jump to the read value.
    JumpNonZeroAA(Pos, Pos),

    /// jmpz [p1], imm2
    JumpZeroAI(Pos, Imm),

    /// jmpz imm1, imm2
    /// If p1 is zero, jump to imm2
    JumpZeroII(Imm, Imm),

    /// jmpz imm1, [p2]
    /// If p1 is non-zero, read value at address imm2. Jump to the read value.
    JumpZeroIA(Imm, Pos),

    /// jmpz [p1], [p2]
    /// If p1 is non-zero, read value at address imm2. Jump to the read value.
    JumpZeroAA(Pos, Pos),

    LessThanAAA(Pos, Pos, Pos),
    LessThanAIA(Pos, Imm, Pos),
    LessThanIAA(Imm, Pos, Pos),
    LessThanIIA(Imm, Imm, Pos),

    EqualsAAA(Pos, Pos, Pos),
    EqualsAIA(Pos, Imm, Pos),
    EqualsIAA(Imm, Pos, Pos),
    EqualsIIA(Imm, Imm, Pos),

    /// halt
    Halt
}

impl Opcode {
    pub fn len(&self) -> usize {
        use Opcode::*;
        match self {
            InA(_)|OutA(_)|OutI(_) => 2,

            JumpNonZeroAI(_,_)|JumpNonZeroII(_,_)|JumpNonZeroIA(_,_)|JumpNonZeroAA(_,_)|
            JumpZeroAI(_,_)   |JumpZeroII(_,_)   |JumpZeroIA(_,_)   |JumpZeroAA(_,_) 
            => 3,

            LessThanAAA(_,_,_)|LessThanAIA(_,_,_)|LessThanIAA(_,_,_)|LessThanIIA(_,_,_)|
            EqualsAAA(_,_,_)  |EqualsAIA(_,_,_)  |EqualsIAA(_,_,_)  |EqualsIIA(_,_,_)  |
            AddAAA(_,_,_)     |AddAIA(_,_,_)     |AddIAA(_,_,_)     |AddIIA(_,_,_)     |
            MulAAA(_,_,_)     |MulAIA(_,_,_)     |MulIAA(_,_,_)     |MulIIA(_,_,_)
            => 4,

            Halt 
            => 1
        }
    }
}

impl Program {
    pub fn from_input(input: &str) -> Program {
        // Remove new lines from input string
        let input = input.replace("\r", "").replace("\n", "");
        
        let memory: Vec<isize> = input.split(',')
                                      // Ignore empty strings from split
                                      .filter(|x| x.len() > 0)
                                      // Parse ints as usize
                                      .map(|x|  x.parse::<isize>().expect(&format!("Error parsing: {}\n", x)))
                                      // Collect into Vec<usize>
                                      .collect();


        // Generate a program converting the input into a Vec<usize>
        Program {
            ip: 0,
            memory: memory,
            instructions: HashMap::new(),
            input: Vec::new(),
            output: Vec::new(),
            halted: false
        }
    }
    
    /// Print the current memory state of the emulator
    pub fn _print(&self) {
        print!("IP: {:06}\n", self.ip);
        let chunk_size = 0x8;
        for (i, bytes) in self.memory.chunks(chunk_size).enumerate() {
            print!("{:06} ", i*chunk_size);
            for b in bytes {
                print!("{:07} ", b);
            }
            print!("\n");
        }
    }

    /// Lift the instruction at the given address. Panics if unknown opcode is found.
    pub fn lift(&mut self, addr: Pos) -> Option<Opcode> {
        let opcode = self.memory[addr];
        // info!("[{}] Lifting\n", addr);

        match opcode {
            00001|01001|00101|01101| // Add
            00002|01002|00102|01102| // Mul
            00007|00107|01007|01107| // LessThan
            00008|00108|01008|01108  // Equals
            => {
                // Lifting an instruction with 3 parameters
                let param1 = self.read(addr+1);
                let param2 = self.read(addr+2);
                let param3 = self.read(addr+3);
                assert!(param3 >= 0);

                let op = match opcode {
                    00001 => Opcode::AddAAA(param1 as usize, param2 as usize, param3 as usize),
                    00002 => Opcode::MulAAA(param1 as usize, param2 as usize, param3 as usize),
                    01001 => Opcode::AddAIA(param1 as usize, param2 as isize, param3 as usize),
                    01002 => Opcode::MulAIA(param1 as usize, param2 as isize, param3 as usize),
                    00101 => Opcode::AddIAA(param1 as isize, param2 as usize, param3 as usize),
                    00102 => Opcode::MulIAA(param1 as isize, param2 as usize, param3 as usize),
                    01101 => Opcode::AddIIA(param1 as isize, param2 as isize, param3 as usize),
                    01102 => Opcode::MulIIA(param1 as isize, param2 as isize, param3 as usize),
                    00007 => Opcode::LessThanAAA(param1 as usize, param2 as usize, param3 as usize),
                    00107 => Opcode::LessThanIAA(param1 as isize, param2 as usize, param3 as usize),
                    01007 => Opcode::LessThanAIA(param1 as usize, param2 as isize, param3 as usize),
                    01107 => Opcode::LessThanIIA(param1 as isize, param2 as isize, param3 as usize),
                    00008 => Opcode::EqualsAAA(param1 as usize, param2 as usize, param3 as usize),
                    00108 => Opcode::EqualsIAA(param1 as isize, param2 as usize, param3 as usize),
                    01008 => Opcode::EqualsAIA(param1 as usize, param2 as isize, param3 as usize),
                    01108 => Opcode::EqualsIIA(param1 as isize, param2 as isize, param3 as usize),
                    _ => unreachable!()
                };
                // debug!("Lifted [{:4}] {:?}\n", addr, op);

                self.instructions.insert(addr, op);
                Some(op)
            }
            003|103| // In
            004|104  // Out
            => {
                // Lifting an instruction with 1 parameter
                let dest = self.read(addr+1);
                assert!(dest >= 0);
                let op = match opcode {
                    003 => Opcode::InA(dest as usize),
                    004 => Opcode::OutA(dest as usize),
                    104 =>  Opcode::OutI(dest as isize),
                    _ => unreachable!()

                };
                self.instructions.insert(addr, op);
                Some(op)
            }
            0005|0105|1005|1105| // JumpNonZero
            0006|0106|1006|1106  // JumpZero
            => {
                // Lifting an instruction with 2 parameters
                let param1 = self.read(addr+1);
                let param2 = self.read(addr+2);

                let op = match opcode {
                    0005 => Opcode::JumpNonZeroAA(param1 as usize, param2 as usize),
                    0105 => Opcode::JumpNonZeroIA(param1 as isize, param2 as usize),
                    1005 => Opcode::JumpNonZeroAI(param1 as usize, param2 as isize),
                    1105 => Opcode::JumpNonZeroII(param1 as isize, param2 as isize),
                    0006 => Opcode::JumpZeroAA(param1 as usize, param2 as usize),
                    0106 => Opcode::JumpZeroIA(param1 as isize, param2 as usize),
                    1006 => Opcode::JumpZeroAI(param1 as usize, param2 as isize),
                    1106 => Opcode::JumpZeroII(param1 as isize, param2 as isize),
                    _ => unreachable!()
                };

                self.instructions.insert(addr, op);
                Some(op)
            }
            10001|10002| 
            11001|11002|
            11101|11102|
            10101|10102 
            => {
                panic!("Read an opcode for immediate in destination.. shouldn't happen!");
            }
            99 => {
                // Lifting an Halt opcode
                self.instructions.insert(addr, Opcode::Halt);
                Some(Opcode::Halt)
            }
            _ => { 
                // Hit an unknown opcode, break out of the loop
                info!("Unknown opcode @ {}: {}\n", addr, opcode);
                None
            }
        }
    }

    /// Execute the current program loaded into the emulator.
    ///
    /// The emulator will see if the current instruction has been lifted already. If not, attempt
    /// to lift the instruction. If so, use the previously lifted instruction.
    pub fn run(&mut self) {
        loop {
            let opcode = self.instructions.get(&self.ip);
            let opcode = match opcode {
                // Haven't seen this opcode yet, attempt to lift it from memory
                None => {
                    match self.lift(self.ip) {
                        Some(op) => op,
                        None => panic!("Failed to lift addr at {}", self.ip)
                    }
                }

                // Seen this opcode already, attempt to emulate it
                Some(op) => { *op }
            };
            info!("Executing: {:?}\n", opcode);
            match opcode {
                Opcode::AddAAA(param1, param2, dest) => {
                    let value1 = self.read(param1);
                    let value2 = self.read(param2);
                    let result = value1 + value2;
                    debug!("AddAAA: {} = {} + {} ({})\n", dest, value1, value2, result);
                    self.write(dest, result);
                    self.ip += 4;
                }
                Opcode::AddIAA(value1, param2, dest) => {
                    let value2 = self.read(param2);
                    let result = value1 + value2;
                    debug!("AddIAA: {} = {} + {} ({})\n", dest, value1, value2, result);
                    self.write(dest, result);
                    self.ip += 4;
                }
                Opcode::AddAIA(param1, value2, dest) => {
                    let value1 = self.read(param1);
                    let result = value1 + value2;
                    debug!("AddIAA: {} = {} + {} ({})\n", dest, value1, value2, result);
                    self.write(dest, result);
                    self.ip += 4;
                }
                Opcode::AddIIA(value1, value2, dest) => {
                    let result = value1 + value2;
                    debug!("AddIIA: {} = {} + {} ({})\n", dest, value1, value2, result);
                    self.write(dest, result);
                    self.ip += 4;
                }
                Opcode::MulAAA(param1, param2, dest) => {
                    let value1 = self.read(param1);
                    let value2 = self.read(param2);
                    let result = value1 * value2;
                    debug!("MulAAA: {} = {} * {} ({})\n", dest, value1, value2, result);
                    self.write(dest, result);
                    self.ip += 4;
                }
                Opcode::MulAIA(param1, value2, dest) => {
                    let value1 = self.read(param1);
                    let result = value1 * value2;
                    debug!("MulAIA: [{}]({}) = [{}]({}) * {}\n", dest, result, param1, value1, value2);
                    self.write(dest, result);
                    self.ip += 4;
                }
                Opcode::MulIAA(value1, param2, dest) => {
                    let value2 = self.read(param2);
                    let result = value1 * value2;
                    debug!("MulIIA: {} = {} * {} ({})\n", dest, value1, value2, result);
                    self.write(dest, result);
                    self.ip += 4;
                }
                Opcode::MulIIA(value1, value2, dest) => {
                    let result = value1 * value2;
                    debug!("MulIIA: {} = {} + {} ({})\n", dest, value1, value2, result);
                    self.write(dest, result);
                    self.ip += 4;
                }
                Opcode::InA(dest) => {
                    let input_val = self.read_input();
                    if input_val.is_none() {
                        // print!("InA without any input.. breaking\n");
                        break;
                    }
                    let input_val = input_val.unwrap();
                    debug!("InA: [{}] = {}\n", dest, input_val);
                    self.write(dest, input_val);
                    self.ip += 2;
                }
                Opcode::OutA(dest) => {
                    let value = self.read(dest);
                    debug!("OutA: output.push({})\n", value);
                    self.write_output(value);
                    self.ip += 2;
                }
                Opcode::OutI(value) => {
                    debug!("OutA: output.push({})\n", value);
                    self.write_output(value);
                    self.ip += 2;
                }
                Opcode::JumpNonZeroII(value1, value2) => {
                    debug!("JumpNonZeroII: if {} is nonzero, jmp to {}\n", value1, value2);
                    if value1 != 0 {
                        debug!("   ip = {}\n", value2);
                        self.ip = value2 as usize;
                    } else {
                        debug!("   ip += 3\n");
                        self.ip += 3;
                    }
                }
                Opcode::JumpNonZeroAI(param1, value2) => {
                    let value1 = self.read(param1);
                    debug!("JumpNonZeroAI: if {} is nonzero, jmp to {}\n", value1, value2);
                    if value1 != 0 {
                        debug!("   ip = {}\n", value2);
                        self.ip = value2 as usize;
                    } else {
                        debug!("   ip += 3\n");
                        self.ip += 3;
                    }
                }
                Opcode::JumpNonZeroIA(value1, param2) => {
                    let value2 = self.read(param2);
                    debug!("JumpNonZeroIA: if {} is nonzero, jmp to {}\n", value1, value2);
                    if value1 != 0 {
                        debug!("   ip = {}\n", value2);
                        self.ip = value2 as usize;
                    } else {
                        debug!("   ip += 3\n");
                        self.ip += 3;
                    }
                }
                Opcode::JumpNonZeroAA(param1, param2) => {
                    let value1 = self.read(param1);
                    let value2 = self.read(param2);
                    debug!("JumpNonZeroIA: if {} is nonzero, jmp to {}\n", value1, value2);
                    if value1 != 0 {
                        debug!("   ip = {}\n", value2);
                        self.ip = value2 as usize;
                    } else {
                        debug!("   ip += 3\n");
                        self.ip += 3;
                    }
                }
                Opcode::JumpZeroII(value1, value2) => {
                    debug!("JumpZeroII: if {} is nonzero, jmp to {}\n", value1, value2);
                    if value1 == 0 {
                        debug!("   ip = {}\n", value2);
                        self.ip = value2 as usize;
                    } else {
                        debug!("   ip += 3\n");
                        self.ip += 3;
                    }
                }
                Opcode::JumpZeroAI(param1, value2) => {
                    let value1 = self.read(param1);
                    debug!("JumpZeroAI: if {} is nonzero, jmp to {}\n", value1, value2);
                    if value1 == 0 {
                        debug!("   ip = {}\n", value2);
                        self.ip = value2 as usize;
                    } else {
                        debug!("   ip += 3\n");
                        self.ip += 3;
                    }
                }
                Opcode::JumpZeroIA(value1, param2) => {
                    let value2 = self.read(param2);
                    debug!("JumpZeroIA: if {} is nonzero, jmp to {}\n", value1, value2);
                    if value1 == 0 {
                        debug!("   ip = {}\n", value2);
                        self.ip = value2 as usize;
                    } else {
                        debug!("   ip += 3\n");
                        self.ip += 3;
                    }
                }
                Opcode::JumpZeroAA(param1, param2) => {
                    let value1 = self.read(param1);
                    let value2 = self.read(param2);
                    debug!("JumpZeroIA: if {} is nonzero, jmp to {}\n", value1, value2);
                    if value1 == 0 {
                        debug!("   ip = {}\n", value2);
                        self.ip = value2 as usize;
                    } else {
                        debug!("   ip += 3\n");
                        self.ip += 3;
                    }
                }
                Opcode::LessThanAAA(param1, param2, dest) => {
                    let value1 = self.read(param1);
                    let value2 = self.read(param2);
                    debug!("LessThanAAA: if {} < {}, [{}] = 1 else [{}] = 0\n", value1, value2, dest, dest);
                    let value = if value1 < value2 { 1 } else { 0 };
                    self.write(dest, value);
                    self.ip += 4;
                }
                Opcode::LessThanIAA(value1, param2, dest) => {
                    let value2 = self.read(param2);
                    debug!("LessThanAAA: if {} < {}, [{}] = 1 else [{}] = 0\n", value1, value2, dest, dest);
                    let value = if value1 < value2 { 1 } else { 0 };
                    self.write(dest, value);
                    self.ip += 4;
                }
                Opcode::LessThanAIA(param1, value2, dest) => {
                    let value1 = self.read(param1);
                    debug!("LessThanAAA: if {} < {}, [{}] = 1 else [{}] = 0\n", value1, value2, dest, dest);
                    let value = if value1 < value2 { 1 } else { 0 };
                    self.write(dest, value);
                    self.ip += 4;
                }
                Opcode::LessThanIIA(value1, value2, dest) => {
                    debug!("LessThanAAA: if {} < {}, [{}] = 1 else [{}] = 0\n", value1, value2, dest, dest);
                    let value = if value1 < value2 { 1 } else { 0 };
                    self.write(dest, value);
                    self.ip += 4;
                }
                Opcode::EqualsAAA(param1, param2, dest) => {
                    let value1 = self.read(param1);
                    let value2 = self.read(param2);
                    debug!("EqualsAAA: if {} == {}, [{}] = 1 else [{}] = 0\n", value1, value2, dest, dest);
                    let value = if value1 == value2 { 1 } else { 0 };
                    self.write(dest, value);
                    self.ip += 4;
                }
                Opcode::EqualsIAA(value1, param2, dest) => {
                    let value2 = self.read(param2);
                    debug!("EqualsAAA: if {} == {}, [{}] = 1 else [{}] = 0\n", value1, value2, dest, dest);
                    let value = if value1 == value2 { 1 } else { 0 };
                    self.write(dest, value);
                    self.ip += 4;
                }
                Opcode::EqualsAIA(param1, value2, dest) => {
                    let value1 = self.read(param1);
                    debug!("EqualsAAA: if {} == {}, [{}] = 1 else [{}] = 0\n", value1, value2, dest, dest);
                    let value = if value1 == value2 { 1 } else { 0 };
                    self.write(dest, value);
                    self.ip += 4;
                }
                Opcode::EqualsIIA(value1, value2, dest) => {
                    debug!("EqualsAAA: if {} == {}, [{}] = 1 else [{}] = 0\n", value1, value2, dest, dest);
                    let value = if value1 == value2 { 1 } else { 0 };
                    self.write(dest, value);
                    self.ip += 4;
                }
                Opcode::Halt => { 
                    self.halted = true;
                    break; 
                }
                // _ => panic!("Cannot execute {:?}", opcode)
            }
        }
    }

    /// Write a value to the given address.
    ///
    /// Since data and code reside in the same memory, a write could corrupt a cached instruction.
    /// On each write, there is a check to see if the write corrupts a cached instruction and if
    /// so, the cached instruction is updated. 
    pub fn write(&mut self, address: Pos, value: Imm) {
        assert!(address <= self.memory.len());
        self.memory[address] = value;

        // A write could overwrite a cached instruction. Check if this write corrupts a previously
        // lifted instruction.
        let mut modified = None;
        for (start, op) in self.instructions.iter() {
            let end = start + op.len();
            if (start..&end).contains(&&address) {
                // Found the instruction that was modified. Mark the instruction address to check.
                modified = Some(start.clone());
                break;
            }
        }

        // If this write, modified an instruction, attempt to lift the new instruction at this address:
        // * If the modified instruction is still a valid instruction, update the cache.
        // * If the modified instruction results in an invalid instruction, invalidate the cache.
        if let Some(start) = modified {
            let new_instr = self.lift(start);
            let old_op = self.instructions.get(&start);
            match new_instr {
                Some(new_op) => {
                    info!("[{}] {:?} -> {:?} -- New instruction\n", start, old_op, new_op);
                    self.instructions.insert(start, new_op);
                }
                None => {
                    info!("[{}] {:?} -> None -- New instruction is invalid\n", start, old_op);
                    self.instructions.remove(&start);
                }
            }
        }
    }

    /// Read a value from the given address
    pub fn read(&mut self, address: Pos) -> Imm {
        assert!(address <= self.memory.len());
        self.memory[address as usize]
    }

    /// Returns the next item in the input buffer
    pub fn read_input(&mut self) -> Option<isize> {
        if self.input.len() == 0 { return None; }
        Some(self.input.remove(0))
    }

    /// Write a value to the output buffer
    pub fn write_output(&mut self, value: isize) {
        self.output.push(value);
        // print!("{}\n", value);
    }

    pub fn _print_output(&self) {
        for o in self.output.iter() {
            print!("{}\n", o);
        }
    }
}


fn stage1(input: &str) {
    let result = [0, 1, 2, 3, 4].iter()
        .permutations(5)
        .map(|sequence| {
            let mut old_result = 0;
            let mut program = Program::from_input(input);
            for s in sequence {
                program.input.push(*s);
                program.input.push(old_result);
                program.run();
                old_result = program.output[0];
                program = Program::from_input(input);
            }
            old_result
        })
        .max().unwrap();

        print!("Stage 1: {:?}\n", result);
}

fn feedback_run(input: &str, sequence: &[&isize]) -> isize {
    let mut cpus = Vec::new();
    for s in sequence.iter() {
        let mut p = Program::from_input(input);
        p.input.push(**s);
        cpus.push(p);
    }

    let mut finished = [false; 5];
    let mut prev_result = 0;
    let mut i = 0;
    loop {
        let seq_num = sequence[i];
        let mut curr_cpu = &mut cpus[i];
        curr_cpu.input.push(prev_result);
        curr_cpu.run();
        prev_result = curr_cpu.output.remove(0);
        finished[i] = curr_cpu.halted;
        if i == 4 && finished.iter().all(|&x| x == true) {
            break;
        }
        i = (i + 1) % 5;
    }

    prev_result
}

fn stage2(input: &str) {
    let result = [9,8,7,6,5].iter()
        .permutations(5)
        .map(|sequence| {
            feedback_run(input, sequence.as_slice())
        })
        .max().unwrap();

        print!("Stage 2: {:?}\n", result);
    
}

fn main() {
    // let input = "3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0";
    let input = include_str!("../input");
    stage1(input);
    stage2(input);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_1() {
        let input = "3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0";
        let sequence = [4,3,2,1,0];
        let mut old_result = 0;
        let mut program = Program::from_input(input);
        for s in &sequence {
            program.input.push(*s);
            program.input.push(old_result);
            program.run();
            old_result = program.output[0];
            program = Program::from_input(input);
        }
        assert_eq!(old_result, 43210);
    }

    #[test]
    fn test_example_2() {
        let input = "3,23,3,24,1002,24,10,24,1002,23,-1,23,101,\
            5,23,23,1,24,23,23,4,23,99,0,0";
        let sequence = [0,1,2,3,4];
        let mut old_result = 0;
        let mut program = Program::from_input(input);
        for s in &sequence {
            program.input.push(*s);
            program.input.push(old_result);
            program.run();
            old_result = program.output[0];
            program = Program::from_input(input);
        }
        assert_eq!(old_result, 54321);
    }

    #[test]
    fn test_example_3() {
        let input = "3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,\
            1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0";
        let sequence = [1,0,4,3,2];
        let mut old_result = 0;
        let mut program = Program::from_input(input);
        for s in &sequence {
            program.input.push(*s);
            program.input.push(old_result);
            program.run();
            old_result = program.output[0];
            program = Program::from_input(input);
        }
        assert_eq!(old_result, 65210);
    }

    #[test]
    fn test_stage2_1() {
        let input = "3,26,1001,26,-4,26,3,27,1002,27,2,27,1,27,26,27,4,27,1001,28,-1,28,1005,28,6,99,0,0,5";
        assert_eq!(feedback_run(&input, &[&9,&8,&7,&6,&5]), 139629729);
    }

    #[test]
    fn test_stage2_2() {
        let input = "3,52,1001,52,-5,52,3,53,1,52,56,54,1007,54,5,55,1005,55,26,1001,54,\
            -5,54,1105,1,12,1,53,54,53,1008,54,0,55,1001,55,1,55,2,53,55,53,4,\
            53,1001,56,-1,56,1005,56,6,99,0,0,0,0,10";
        assert_eq!(feedback_run(&input, &[&9,&7,&8,&5,&6]), 18216);
    }
}
