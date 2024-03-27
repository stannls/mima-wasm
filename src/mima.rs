use wasm_bindgen::prelude::*;

const MEMORY_SIZE: usize = 1048576;
const VALUE_SIZE: usize = 16777216;
const MINUS_ONE: usize = 0b100000000000000000000000;

#[wasm_bindgen]
pub struct Mima {
    akku: usize,
    iar: usize,
    halt: bool,
    memory: [usize; MEMORY_SIZE],
}

#[wasm_bindgen]
pub struct MimaDebug {
    pub akku: usize,
    pub iar: usize,
    pub halt: bool,
}

#[wasm_bindgen]
impl Mima {
    pub fn write_adress(&mut self, adress: usize, value: usize) -> bool {
        if adress >= MEMORY_SIZE || value >= VALUE_SIZE {
            false
        } else {
            self.memory[adress] = value;
            true
        }
    }

    pub fn read_adress(&mut self, adress: usize) -> Option<usize> {
        if adress >= MEMORY_SIZE {
            None
        } else {
            Some(self.memory[adress])
        }
    }

    pub fn memdump(&mut self) -> Vec<usize> {
        Vec::from(self.memory)
    }

    pub fn step(&mut self) {
        if self.halt {
            return;
        }
        let command = Command::from_usize(self.memory[self.iar]);
        if command.is_none() || command.as_ref().unwrap().value >= MEMORY_SIZE {
            self.halt = true;
            return;
        }
        let command = command.unwrap();
        let mut next_instruction = self.iar + 1;
        match command.instruction {
            Instruction::LDC => self.akku = command.value,
            Instruction::LDV => self.akku = self.memory[command.value],
            Instruction::STV => self.memory[command.value] = self.akku,
            // TODO: Overflow checking
            Instruction::ADD => self.akku += self.memory[command.value],
            Instruction::AND => self.akku &= self.memory[command.value],
            Instruction::OR => self.akku |= self.memory[command.value],
            Instruction::XOR => self.akku ^= self.memory[command.value],
            Instruction::EQL => {
                self.akku = if self.akku == self.memory[command.value] {
                    MINUS_ONE
                } else {
                    0
                }
            }
            Instruction::JMP => next_instruction = command.value,
            Instruction::JMN => {
                if self.akku >= MINUS_ONE {
                    next_instruction = command.value
                }
            }
            Instruction::LDIV => self.akku = self.memory[self.memory[command.value]],
            Instruction::STIV => self.memory[self.memory[command.value]] = self.akku,
            Instruction::HLT => self.halt = true,
            Instruction::NOT => self.akku = !self.akku,
            Instruction::RAR => self.akku = self.akku.rotate_right(1),
        }
        if !self.halt {
            self.iar = next_instruction;
        }
    }
    pub fn run(&mut self) {
        while !self.halt {
            self.step();
        }
    }
    pub fn new() -> Mima {
        Mima {
            akku: 0,
            iar: 0,
            halt: false,
            memory: [0; MEMORY_SIZE],
        }
    }
    pub fn load(&mut self, program: Vec<Command>) -> bool {
        if program.len() >= MEMORY_SIZE {
            return false;
        }
        for i in 0..program.len() {
            self.memory[i] = program[i].to_usize();
        }
        true
    }
    pub fn get_debug(&self) -> MimaDebug {
        MimaDebug {
            akku: self.akku,
            iar: self.iar,
            halt: self.halt,
        }
    }
    pub fn get_next_instruction(&self) -> Option<Command> {
        Command::from_usize(self.memory[self.iar])
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Instruction {
    LDC,
    LDV,
    STV,
    ADD,
    AND,
    OR,
    XOR,
    EQL,
    JMP,
    JMN,
    LDIV,
    STIV,
    NOT,
    RAR,
    HLT,
}

impl Instruction {
    pub fn from_opcode(opcode: usize) -> Option<Instruction> {
        match opcode {
            0 => Some(Instruction::LDC),
            1 => Some(Instruction::LDV),
            2 => Some(Instruction::STV),
            3 => Some(Instruction::ADD),
            4 => Some(Instruction::AND),
            5 => Some(Instruction::OR),
            6 => Some(Instruction::XOR),
            7 => Some(Instruction::EQL),
            8 => Some(Instruction::JMP),
            9 => Some(Instruction::JMN),
            10 => Some(Instruction::LDIV),
            11 => Some(Instruction::STIV),
            240 => Some(Instruction::HLT),
            241 => Some(Instruction::NOT),
            242 => Some(Instruction::RAR),
            _ => None,
        }
    }
    pub fn to_opcode(&self) -> usize {
        match self {
            Instruction::LDC => 0,
            Instruction::LDV => 1,
            Instruction::STV => 2,
            Instruction::ADD => 3,
            Instruction::AND => 4,
            Instruction::OR => 5,
            Instruction::XOR => 6,
            Instruction::EQL => 7,
            Instruction::JMP => 8,
            Instruction::JMN => 9,
            Instruction::LDIV => 10,
            Instruction::STIV => 11,
            Instruction::HLT => 240,
            Instruction::NOT => 241,
            Instruction::RAR => 242,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq)]
pub struct Command {
    pub instruction: Instruction,
    pub value: usize,
}

#[wasm_bindgen]
impl Command {
    pub fn from_usize(value: usize) -> Option<Command> {
        if value > VALUE_SIZE {
            None
        } else {
            // Convert the 4 most significant bits into opcode
            let opcode: usize = (20..24)
                .map(|n| (value >> n) & 1)
                .enumerate()
                .fold(0, |acc, (index, elem)| {
                    acc + elem * (2 as usize).pow(index as u32)
                });
            dbg!(opcode);
            // Convert the 20 least significant bits into argument
            let value: usize = (0..20)
                .map(|n| (value >> n) & 1)
                .enumerate()
                .fold(0, |acc, (index, elem)| {
                    acc + elem * (2 as usize).pow(index as u32)
                });
            dbg!(value);
            Instruction::from_opcode(opcode).map(|instruction| Command { instruction, value })
        }
    }
    pub fn to_usize(&self) -> usize {
        let opcode = self.instruction.to_opcode();
        let opcode_bytes = (0..4).map(|n| (opcode >> n) & 1).rev();
        let value_bytes = (0..20).map(|n| (self.value >> n) & 1).rev();
        opcode_bytes
            .chain(value_bytes)
            .rev()
            .enumerate()
            .fold(0, |acc, (index, elem)| {
                acc + elem * (2 as usize).pow(index as u32)
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::mima::{Command, Instruction};

    #[test]
    fn command_loading() {
        //TODO: Expand tests
        let testcode = 0b000100000000000000000001;
        let cmd = Command::from_usize(testcode);

        assert_eq!(
            cmd,
            Some(Command {
                instruction: Instruction::LDV,
                value: 1
            })
        );
    }

    #[test]
    fn command_dumping() {
        let testcode = 0b000100000000000000000001;
        let cmd = Command {
            instruction: Instruction::LDV,
            value: 1,
        };

        assert_eq!(cmd.to_usize(), testcode);
    }
}
