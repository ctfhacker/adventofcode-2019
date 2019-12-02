#[macro_use]
extern crate log;

use std::collections::HashMap;

#[derive(Debug, Clone)]
/// Program struct containing the current state of the emulator
struct Program {
    /// Instruction Pointer
    ip: usize,

    /// Current memory in the emulator
    memory: Vec<usize>,

    /// Lifted instructions to be executed in the emulator
    /// HashMap is keyed by IP of the instruction
    instructions: HashMap<usize, Opcode>
}

/// Available opcodes in our computer emulator
#[derive(Clone, Copy, Debug)]
enum Opcode {
    Add(usize, usize, usize),
    Mul(usize, usize, usize),
    Halt
}

impl Program {
    pub fn from_input(input: &str) -> Program {
        // Remove new lines from input string
        let input = input.replace("\r", "").replace("\n", "");
        
        let memory: Vec<usize> = input.split(',')
                                      // Ignore empty strings from split
                                      .filter(|x| x.len() > 0)
                                      // Parse ints as usize
                                      .map(|x|  x.parse::<usize>().unwrap())
                                      // Collect into Vec<usize>
                                      .collect();


        // Generate a program converting the input into a Vec<usize>
        Program {
            ip: 0,
            memory: memory,
            instructions: HashMap::new()
        }
    }
    
    /// Set address 1 to noun and address 2 to verb as per the Alarm State
    ///
    /// The inputs should still be provided to the program by replacing the values at addresses 1 and 2, 
    /// just like before. In this program, the value placed in address 1 is called the noun, and the value 
    /// placed in address 2 is called the verb. 
    pub fn set_alarm_state(&mut self, noun: usize, verb: usize) {
        self.write(1, noun);
        self.write(2, verb);
    }

    /// Print the current memory state of the emulator
    pub fn print(&self) {
        print!("IP: {:06}\n", self.ip);
        let chunk_size = 0x8;
        for (i, bytes) in self.memory.chunks(chunk_size).enumerate() {
            print!("{:06} ", i*chunk_size);
            for b in bytes {
                print!("{:07x} ", b);
            }
            print!("\n");
        }
    }

    /// Lift the instruction at the given address. Panics if unknown opcode is found.
    pub fn lift(&mut self, addr: usize) -> Opcode {
        let opcode = self.memory[addr];
        match opcode {
            1|2 => {
                // Lifting an Add or Mul opcode
                let param1 = self.read(addr+1);
                let param2 = self.read(addr+2);
                let dest = self.read(addr+3);
                let op = if opcode == 1 {
                    Opcode::Add(param1, param2, dest)
                } else {
                    Opcode::Mul(param1, param2, dest)
                };
                debug!("Lifted [{:4}] {:?}\n", addr, op);

                /*
                // Self modifying code check here
                if self.instructions.contains_key(addr) && self.instructions.get(addr) != op {
                    panic!("Already different lifted instruction at {} before: {} after: {}", 
                        addr, self.instructions.get(addr), op);
                }
                */

                self.instructions.insert(addr, op);
                op
            }
            99 => {
                // Lifting an Halt opcode
                self.instructions.insert(addr, Opcode::Halt);
                Opcode::Halt
            }
            _ => { 
                // Hit an unknown opcode, break out of the loop
                panic!("Unknown instruction @ {}\n", addr);
            }
        }
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.instructions.get(&self.ip);
            let opcode = match opcode {
                // Haven't seen this opcode yet, attempt to lift it from memory
                None => self.lift(self.ip),

                // Seen this opcode already, attempt to emulate it
                Some(op) => *op,
            };
            match opcode {
                Opcode::Add(param1, param2, dest) => {
                    let value1 = self.read(param1);
                    let value2 = self.read(param2);
                    self.write(dest, value1 + value2);
                    self.ip += 4;
                }
                Opcode::Mul(param1, param2, dest) => {
                    let value1 = self.read(param1);
                    let value2 = self.read(param2);
                    self.write(dest, value1 * value2);
                    self.ip += 4;
                }
                Opcode::Halt => { break; }
            }
        }
    }

    /// Write a value to the given address
    pub fn write(&mut self, address: usize, value: usize) {
        assert!(address <= self.memory.len());
        self.memory[address] = value;
    }

    /// Read a value from the given address
    pub fn read(&mut self, address: usize) -> usize {
        assert!(address <= self.memory.len());
        self.memory[address]
    }
}

/// Test emuation of the alarm state of 1202
fn stage1(input: &str) {
    let mut program = Program::from_input(input);
    // Set program alarm state to 1202
    program.set_alarm_state(12, 2);
    program.run();
    print!("Stage 1: {}\n", program.read(0));
}

/// Brute force the alarm state for our wanted output
fn stage2(input: &str) {
    let program = Program::from_input(input);
    for noun in 0..100 {
        for verb in 0..100 {
            let mut curr_program = program.clone();
            curr_program.set_alarm_state(noun, verb);
            curr_program.run();
            if curr_program.read(0) == 19690720 {
                curr_program.print();
                print!("Stage 2: {}\n", noun * 100 + verb);
                break;
            }
        }
    }
}

fn main() {
    let input = include_str!("../input");
    stage1(input);
    stage2(input);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_program() {
        let input = "1,9,10,3,2,3,11,0,99,30,40,50";
        let mut program = Program::from_input(input);
        program.run();
        assert_eq!(program.memory[0], 3500);
    }
}
