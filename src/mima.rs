use std::isize;
use wasm_bindgen::prelude::*;

const  MEMORY_SIZE: usize = 1048576;
const  VALUE_SIZE: usize = 16777216;

#[wasm_bindgen]
pub struct Mima {
    akku: usize,
    iar: usize,
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
        let instruction = self.memory[self.iar];
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub enum Instruction {
   LDC,
   LDV,
   STV,
   LDIV,
   STIV,
   ADD,
   AND,
   OR,
   XOR,
   NOT,
   RAR,
   EQL,
   JMP,
   JNN,
   HLT
}

#[wasm_bindgen]
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
            let opcode: Vec<usize> = (0..4).map(|n| (value >> n) & 1).collect();
            let argument: Vec<usize> = (4..24).map(|n| (value >> n) & 1).collect();
            dbg!(opcode);
            dbg!(argument);
            match opcode {
                _ => None
            }
        }
    }
}
