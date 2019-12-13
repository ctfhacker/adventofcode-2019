use std::collections::HashMap;

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
        if LOGLEVEL >= 1 {
            print!("INFO:  ");
            print!($($arg)*);
        }
    }
}

// Immediate parameter
type Imm = isize;

// Position parameter
type Pos = usize;

// Relative parameter
// type Rel = isize;

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
    halted: bool,
    
    /// Current relative address
    relative_base: isize
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Mode {
    Positional(usize),
    Immediate(isize),
    Relative(isize)
}

impl std::fmt::Debug for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Positional(addr) => write!(f, "Pos({})", addr),
            Mode::Immediate(imm) => write!(f, "Imm({})", imm),
            Mode::Relative(rel) => write!(f, "Rel({})", rel),
        }
    }
}

use Mode::*;

/// Available opcodes in our computer emulator
/// 
/// Each opcode is appended by how the parameters should be interpretted
///
/// Example:
/// AddPPP - add where all parameters are positions in memory 
/// AddIIP - add where the two parameters are immediates and the result is a position
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Opcode {
    Add(Mode, Mode, Mode),
    Mul(Mode, Mode, Mode),
    In(Mode),
    Out(Mode),
    JumpNonZero(Mode, Mode),
    JumpZero(Mode, Mode),
    LessThan(Mode, Mode, Mode),
    Equals(Mode, Mode, Mode),
    AdjustRelativeBase(Mode),
    Halt
}

impl Opcode {
    /// Returns the length of the instruction.
    ///
    /// This function is used during the instruction caching in order to determine if a given write
    /// destination is in an already lifted instruction.
    pub fn len(&self) -> usize {
        use Opcode::*;
        match self {
            In(_)|Out(_)|AdjustRelativeBase(_) => 2,
            JumpNonZero(_,_)|JumpZero(_,_) => 3,
            LessThan(_,_,_)|Equals(_,_,_)|Add(_,_,_)|Mul(_,_,_) => 4,
            Halt => 1
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
            halted: false,
            relative_base: 0
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
        let mut opcode = self.memory[addr];
        debug!("[{}] Lifting {:05} ", addr, opcode);
        let mode3 = opcode / 10000;
        opcode = opcode % 10000;
        let mode2 = opcode / 1000;
        opcode = opcode % 1000;
        let mode1 = opcode / 100;
        opcode = opcode % 100;
        debug!("{} ({} {} {})\n", opcode, mode3, mode2, mode1);

        match opcode {
            1|2|7|8 => {
                // Lifting an Add, Mul, LessThan, Equals
                let param1 = self.read(addr+1);
                let param2 = self.read(addr+2);
                let param3 = self.read(addr+3);

                let param1 = match mode1 {
                    0 => Positional(param1 as usize),
                    1 => Immediate(param1),
                    2 => Relative(param1),
                    _ => unreachable!()
                };

                let param2 = match mode2 {
                    0 => Positional(param2 as usize),
                    1 => Immediate(param2),
                    2 => Relative(param2),
                    _ => unreachable!()
                };

                let param3 = match mode3 {
                    0 => Positional(param3 as usize),
                    1 => Immediate(param3),
                    2 => Relative(param3),
                    _ => unreachable!()
                };

                let op = match opcode {
                    1 => Opcode::Add(param1, param2, param3),
                    2 => Opcode::Mul(param1, param2, param3),
                    7 => Opcode::LessThan(param1, param2, param3),
                    8 => Opcode::Equals(param1, param2, param3),
                    _ => unreachable!()
                };

                debug!("Lifted [{:4}] {} {:?}\n", addr, opcode, op);

                self.instructions.insert(addr, op);
                Some(op)
            }
            3|4|9 => {
                // Lifting an In, Out, AdjustRelativeBase
                let param1 = self.read(addr+1);
                let value = match mode1 {
                    0 => Positional(param1 as usize),
                    1 => Immediate(param1),
                    2 => Relative(param1),
                    _ => unreachable!()
                };

                let op = match opcode {
                    3 => Opcode::In(value),
                    4 => Opcode::Out(value),
                    9 => Opcode::AdjustRelativeBase(value),
                    _ => unreachable!()
                };

                self.instructions.insert(addr, op);
                Some(op)
            }

            5|6 => {
                // Lifting an JumpNonZero, JumpZero
                let param1 = self.read(addr+1);
                let param2 = self.read(addr+2);

                let param1 = match mode1 {
                    0 => Positional(param1 as usize),
                    1 => Immediate(param1),
                    2 => Relative(param1),
                    _ => unreachable!()
                };

                let param2 = match mode2 {
                    0 => Positional(param2 as usize),
                    1 => Immediate(param2),
                    2 => Relative(param2),
                    _ => unreachable!()
                };

                let op = match opcode {
                    5 => Opcode::JumpNonZero(param1, param2),
                    6 => Opcode::JumpZero(param1, param2),
                    _ => unreachable!()
                };

                self.instructions.insert(addr, op);
                Some(op)
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
                Opcode::Add(param1, param2, dest) => {
                    let value1 = match param1 {
                        Positional(addr) => self.read(addr),
                        Immediate(imm) => imm,
                        Relative(rel_offset) => self.read((self.relative_base + rel_offset) as usize)
                    };
                    let value2 = match param2 {
                        Positional(addr) => self.read(addr),
                        Immediate(imm) => imm,
                        Relative(rel_offset) => self.read((self.relative_base + rel_offset) as usize)
                    };
                    let dest = match dest {
                        Positional(addr) => addr as usize,
                        Immediate(_imm) => panic!("Cannot execute Add with an immediate dest"),
                        Relative(rel_offset) => (self.relative_base + rel_offset) as usize
                    };

                    let result = value1 + value2;
                    debug!("Add: {} = {} + {} ({})\n", dest, value1, value2, result);
                    self.write(dest, result);
                    self.ip += 4;
                }
                Opcode::Mul(param1, param2, dest) => {
                    let value1 = match param1 {
                        Positional(addr) => self.read(addr),
                        Immediate(imm) => imm,
                        Relative(rel_offset) => self.read((self.relative_base + rel_offset) as usize)
                    };
                    let value2 = match param2 {
                        Positional(addr) => self.read(addr),
                        Immediate(imm) => imm,
                        Relative(rel_offset) => self.read((self.relative_base + rel_offset) as usize)
                    };
                    let dest = match dest {
                        Positional(addr) => addr as usize,
                        Immediate(_imm) => panic!("Cannot execute Mul with an immediate dest"),
                        Relative(rel_offset) => (self.relative_base + rel_offset) as usize
                    };

                    let result = value1 * value2;
                    debug!("Mul: [{}] = {} * {} ({})\n", dest, value1, value2, result);
                    self.write(dest, result);
                    self.ip += 4;
                }
                
                Opcode::In(dest) => {
                    let input_val = self.read_input();
                    if input_val.is_none() {
                        // print!("InP without any input.. breaking\n");
                        break;
                    }

                    let dest = match dest {
                        Positional(addr) => addr as usize,
                        Immediate(_imm) => panic!("Cannot execute In with an immediate dest"),
                        Relative(rel_offset) => (self.relative_base + rel_offset) as usize
                    };

                    let input_val = input_val.unwrap();
                    info!("In: [{}] = {}\n", dest, input_val);
                    self.write(dest, input_val);
                    self.ip += 2;
                }

                Opcode::Out(value) => {
                    let value = match value {
                        Positional(addr) => self.read(addr),
                        Immediate(imm) => imm,
                        Relative(rel_offset) => self.read((self.relative_base + rel_offset) as usize)
                    };

                    debug!("Out: output.push({})\n", value);
                    self.write_output(value);
                    self.ip += 2;
                }

                Opcode::JumpNonZero(param1, param2) => {
                    let value1 = match param1 {
                        Positional(addr) => self.read(addr),
                        Immediate(imm) => imm,
                        Relative(rel_offset) => self.read((self.relative_base + rel_offset) as usize)
                    };
                    let value2 = match param2 {
                        Positional(addr) => self.read(addr),
                        Immediate(imm) => imm,
                        Relative(rel_offset) => self.read((self.relative_base + rel_offset) as usize)
                    };
                    debug!("JumpNonZero: if {} is nonzero, jmp to {}\n", value1, value2);
                    if value1 != 0 {
                        debug!("   ip = {}\n", value2);
                        self.ip = value2 as usize;
                    } else {
                        debug!("   ip += 3\n");
                        self.ip += 3;
                    }
                }

                Opcode::JumpZero(param1, param2) => {
                    let value1 = match param1 {
                        Positional(addr) => self.read(addr),
                        Immediate(imm) => imm,
                        Relative(rel_offset) => self.read((self.relative_base + rel_offset) as usize)
                    };
                    let value2 = match param2 {
                        Positional(addr) => self.read(addr),
                        Immediate(imm) => imm,
                        Relative(rel_offset) => self.read((self.relative_base + rel_offset) as usize)
                    };
                    debug!("JumpZero: if {} is nonzero, jmp to {}\n", value1, value2);
                    if value1 == 0 {
                        debug!("   ip = {}\n", value2);
                        self.ip = value2 as usize;
                    } else {
                        debug!("   ip += 3\n");
                        self.ip += 3;
                    }
                }

                Opcode::LessThan(param1, param2, dest) => {
                    let value1 = match param1 {
                        Positional(addr) => self.read(addr),
                        Immediate(imm) => imm,
                        Relative(rel_offset) => self.read((self.relative_base + rel_offset) as usize)
                    };
                    let value2 = match param2 {
                        Positional(addr) => self.read(addr),
                        Immediate(imm) => imm,
                        Relative(rel_offset) => self.read((self.relative_base + rel_offset) as usize)
                    };
                    let dest = match dest {
                        Positional(addr) => addr as usize,
                        Immediate(_imm) => panic!("Cannot execute LessThan with an immediate dest"),
                        Relative(rel_offset) => (self.relative_base + rel_offset) as usize
                    };

                    debug!("LessThan: if {} < {}, [{}] = 1 else [{}] = 0\n", value1, value2, dest, dest);
                    let value = if value1 < value2 { 1 } else { 0 };
                    self.write(dest, value);
                    self.ip += 4;
                }

                Opcode::Equals(param1, param2, dest) => {
                    let value1 = match param1 {
                        Positional(addr) => self.read(addr),
                        Immediate(imm) => imm,
                        Relative(rel_offset) => self.read((self.relative_base + rel_offset) as usize)
                    };
                    let value2 = match param2 {
                        Positional(addr) => self.read(addr),
                        Immediate(imm) => imm,
                        Relative(rel_offset) => self.read((self.relative_base + rel_offset) as usize)
                    };
                    let dest = match dest {
                        Positional(addr) => addr as usize,
                        Immediate(_imm) => panic!("Cannot execute Equals with an immediate dest"),
                        Relative(rel_offset) => (self.relative_base + rel_offset) as usize
                    };

                    debug!("Equals: if {} == {}, [{}] = 1 else [{}] = 0\n", value1, value2, dest, dest);
                    let value = if value1 == value2 { 1 } else { 0 };
                    self.write(dest, value);
                    self.ip += 4;
                }
                Opcode::AdjustRelativeBase(offset) => {
                    let offset = match offset {
                        Positional(addr) => self.read(addr),
                        Immediate(imm) => imm,
                        Relative(rel_offset) => self.read((self.relative_base + rel_offset) as usize)
                    };

                    info!("New relative base: {} = {} + {}\n", self.relative_base + offset, 
                        self.relative_base, offset);
                    self.relative_base += offset; 
                    self.ip += 2;
                }
                Opcode::Halt => { 
                    self.halted = true;
                    break; 
                }
            }
        }
    }

    /// Write a value to the given address.
    ///
    /// Since data and code reside in the same memory, a write could corrupt a cached instruction.
    /// On each write, there is a check to see if the write corrupts a cached instruction and if
    /// so, the cached instruction is updated. 
    pub fn write(&mut self, address: Pos, value: Imm) {
        if address > self.memory.len() {
            debug!("Resizing to {}\n", address + 1000);
            self.memory.resize(address + 1000, 0);
        }
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
        if address > self.memory.len() {
            debug!("Resizing to {}\n", address + 1000);
            self.memory.resize(address + 1000, 0);
        }
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
    }

    pub fn _print_output(&self) {
        for o in self.output.iter() {
            print!("{}\n", o);
        }
    }
}

enum Direction {
    Up,
    Down,
    Left, 
    Right
}

enum Turn {
    Left,
    Right
}

impl From<isize> for Turn {
    fn from(val: isize) -> Turn {
        match val {
            0 => Turn::Left,
            1 => Turn::Right,
            _ => unreachable!()
        }
    }
}

fn main() {
    let input = include_str!("../input");
    let mut program = Program::from_input(input);
    let mut direction = Direction::Up;
    let mut location_x = 0;
    let mut location_y = 0;
    let mut visited: HashMap<(isize, isize), isize> = HashMap::new();
    visited.insert((0, 0), 1);

    loop {
        // If we have already visited a location, use the color printed. Otherwise,
        // the default board is always black
        let curr_location = (location_x, location_y);
        let input_val = visited.get(&curr_location).or(Some(&0)).unwrap();

        program.input.push(*input_val);
        program.run();
        if program.halted { break; }
 
        let color = program.output.remove(0);
        let new_dir: Turn = program.output.remove(0).into();

        visited.insert(curr_location, color);

        // Adjust the direction of the robot based on the output turn
        direction = match (new_dir, direction) {
            (Turn::Left, Direction::Up) => Direction::Left,        
            (Turn::Left, Direction::Down) => Direction::Right,        
            (Turn::Left, Direction::Left) => Direction::Down,        
            (Turn::Left, Direction::Right) => Direction::Up,        
            (Turn::Right, Direction::Up) => Direction::Right,        
            (Turn::Right, Direction::Down) => Direction::Left,        
            (Turn::Right, Direction::Left) => Direction::Up,        
            (Turn::Right, Direction::Right) => Direction::Down,        
        };

        // Adjust the robot location
        match direction {
            Direction::Up => location_y += 1,
            Direction::Down => location_y -= 1,
            Direction::Left => location_x -= 1,
            Direction::Right => location_x += 1
        }
    }

    // Calculate the width of the board needed
    let max_x = visited.keys().map(|x| (x.0 as isize).abs() as usize).max().unwrap();
    let max_y = visited.keys().map(|x| (x.1 as isize).abs() as usize).max().unwrap();

    // Allocate the print buffer with '.' as blank spaces
    let mut buffer = Vec::new();
    for _ in 0..((max_x * 2 + 1) * (max_y * 2 + 1)) {
        buffer.push('.');
    }

    // Set the center of the grid
    let center_x = max_x as usize;
    let center_y = max_y as usize;

    // 
    for ((x, y), &color) in visited.iter() {
        if color == 0 {
            continue;
        }
        let curr_y = center_y + (-1 * *y as isize) as usize;
        let curr_x = (center_x as isize + x) as usize;
        buffer[curr_y * (max_x * 2) + curr_x] = '%';
    }

    // Print the resulting board
    for line in buffer.as_slice().chunks(max_x * 2) {
        for c in line {
            print!("{}", c);
        }
            print!("\n");
    }
}

