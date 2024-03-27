use wasm_bindgen::prelude::*;

use crate::compiler::CompilerOutput;

const MEMORY_SIZE: usize = 1048576;
const VALUE_SIZE: usize = 16777216;
const MINUS_ONE: usize = 0b100000000000000000000000;

#[wasm_bindgen]
pub struct Mima {
    akku: usize,
    iar: usize,
    halt: bool,
    memory: Vec<usize>,
}

#[wasm_bindgen]
pub struct MimaDebug {
    pub akku: usize,
    pub iar: usize,
    pub halt: bool,
}

#[wasm_bindgen]
impl Mima {
    pub fn reset(&mut self) {
        self.akku = 0;
        self.iar = 0;
        self.halt = false;
        self.memory = vec![0; MEMORY_SIZE];
    }
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
        self.memory.to_owned()
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
            Instruction::STIV => {
                let adress = self.memory[command.value];
                self.memory[adress] = self.akku;
            },
            Instruction::HALT => self.halt = true,
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
            memory: vec![0; MEMORY_SIZE],
        }
    }
    pub fn load(&mut self, program: CompilerOutput) -> bool {
        self.reset();
        let code = program.get_mima_code();
        if code.len() >= MEMORY_SIZE {
            return false;
        }
        for i in 0..code.len() {
            self.memory[i] = code[i];
        }
        self.iar = program.get_start_adress();
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
    HALT,
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
            15 => Some(Instruction::HALT),
            241 => Some(Instruction::NOT),
            242 => Some(Instruction::RAR),
            _ => None,
        }
    }
    pub fn from_string(string: &str) -> Option<Instruction> {
        match string {
            "LDC" => Some(Self::LDC),
            "LDV" => Some(Self::LDV),
            "STV" => Some(Self::STV),
            "ADD" => Some(Self::ADD),
            "AND" => Some(Self::AND),
            "OR" => Some(Self::OR),
            "XOR" => Some(Self::XOR),
            "EQL" => Some(Self::EQL),
            "JMP" => Some(Self::JMP),
            "JMN" => Some(Self::JMN),
            "LDIV" => Some(Self::LDIV),
            "STIV" => Some(Self::STIV),
            "HALT" => Some(Self::HALT),
            "NOT" => Some(Self::NOT),
            "RAR" => Some(Self::RAR),
            _ => None
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
            Instruction::HALT => 15,
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
            // Convert the 20 least significant bits into argument
            let value: usize = (0..20)
                .map(|n| (value >> n) & 1)
                .enumerate()
                .fold(0, |acc, (index, elem)| {
                    acc + elem * (2 as usize).pow(index as u32)
                });
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
    use crate::{compiler::CompilerOutput, mima::{Command, Instruction}};

    use super::Mima;

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
    #[test]
    fn mima_add_program() {
        let ldv = Command {instruction: crate::mima::Instruction::LDV, value: 0};
        let add = Command {instruction: crate::mima::Instruction::ADD, value: 1};
        let stv = Command {instruction: crate::mima::Instruction::STV, value: 2};
        let halt = Command {instruction: crate::mima::Instruction::HALT, value: 0};
        let mima_code = vec![22, 20, 0, ldv.to_usize(), add.to_usize(), stv.to_usize(), halt.to_usize()];
        let compiler_output = CompilerOutput::new(mima_code, 3);

        let mut mima = Mima::new();
        mima.load(compiler_output);
        mima.run();
        assert_eq!(mima.halt, true);
        assert_eq!(mima.akku, 42);
        assert_eq!(mima.iar, 6);
        assert_eq!(mima.read_adress(2), Some(42));
    }
}
