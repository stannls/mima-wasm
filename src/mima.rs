use wasm_bindgen::prelude::*;

const  MEMORY_SIZE: usize = 1048576;
const  VALUE_SIZE: usize = 16777216;
const MINUS_ONE: usize = 0b100000000000000000000000;

#[wasm_bindgen]
pub struct Mima {
    akku: usize,
    iar: usize,
    halt: bool,
    memory: [usize; MEMORY_SIZE]
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
            Instruction::EQL => self.akku = if self.akku == self.memory[command.value] {MINUS_ONE} else {0},
            Instruction::JMP => next_instruction = command.value,
            Instruction::JMN => if self.akku >= MINUS_ONE {next_instruction = command.value},
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
   HLT
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
            _ => None
        }
    }
}


#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq)]
pub struct Command{
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
            let opcode: usize = (20..24).map(|n| (value >> n) & 1).rev().fold(0, |acc, elem| elem * (2 as usize).pow(acc as u32));
            // Convert the 20 least significant bits into argument
            let value: usize = (0..20).map(|n| (value >> n) & 1).rev().fold(0, |acc, elem| elem * (2 as usize).pow(acc as u32));
            Instruction::from_opcode(opcode).map(|instruction| Command { instruction, value })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mima::{Command, Instruction};


    #[test]
    fn command_loading() {
        //TODO: Expand tests
        let testcode = 0b000100000000000000000001;
        dbg!(testcode);
        let cmd = Command::from_usize(testcode);

        assert_eq!(cmd, Some(Command {instruction: Instruction::LDV, value: 1}));
    }
}
