use regex::Regex;
use lazy_static::lazy_static;

use wasm_bindgen::prelude::*;

use crate::mima::Instruction;
use crate::mima::Command;

lazy_static! {
    static ref VARIABLE_REGEX: Regex = Regex::new(r"([a-zA-Z]+): DS( ([0-9]+))?").unwrap();
    static ref INSTRUCTION_REGEX: Regex = Regex::new(r"\s*(([a-zA-Z]+):)?\s*([a-zA-Z]+)( ([0-9]+|[a-zA-Z]+))?").unwrap();
}

// Struct reprasantation of the compiler output
#[wasm_bindgen]
pub struct CompilerOutput {
    mima_code: Vec<usize>,
    start_adress: usize,
}

// We can't make the attributes public because of wasm and need to manually write getters.
#[wasm_bindgen]
impl CompilerOutput {
   pub fn get_mima_code(&self) -> Vec<usize> {
        self.mima_code.to_owned()
   }
   pub fn get_start_adress(&self) -> usize {
        self.start_adress.to_owned()
   }
   pub fn new(mima_code: Vec<usize>, start_adress: usize) -> CompilerOutput {
        CompilerOutput { mima_code, start_adress }
   }
}

// Struct representing the step between parsing and generating assembly code
struct ParsedProgram {
    pub variables: Vec<Variable>,
    pub commands: Vec<Cmd>,
}


/*
 * This is a very basic compiler. That is currently wip.
 * For now it only supports basic variable assignments and instructions.
 */
#[wasm_bindgen]
pub fn compile(input: &str) -> Option<CompilerOutput> {
    let parsed = parse_assembly(input)?;
    generate_machinecode(&parsed)
    
}

fn parse_assembly(input: &str) -> Option<ParsedProgram> {
    let mut variables: Vec<Variable> = vec![];
    let mut commands: Vec<Cmd> = vec![];
    let lines: Vec<&str> = input.split("\n").filter(|line| !line.starts_with(";")).filter(|line| !line.is_empty()).collect();
    for line in lines {
        if VARIABLE_REGEX.is_match(line) {
            let captures = VARIABLE_REGEX.captures(line).unwrap();
            let name = captures.get(1).unwrap().as_str();
            let value = captures.get(3).map(|f| f.as_str().parse::<usize>().ok()).flatten();
            variables.push(Variable { name: name.to_string(), value })
        }  else if INSTRUCTION_REGEX.is_match(line) {
            let captures = INSTRUCTION_REGEX.captures(line).unwrap();
            let name = captures.get(3).unwrap().as_str();
            let value = captures.get(5);
            let label = captures.get(2).map(|f| f.as_str().to_string());
            let param = match value {
                Some(value) => match value.as_str().parse::<usize>() {
                    Ok(number) => Param::Fixed(number),
                    Err(_) => Param::Reference(value.as_str().to_string())
                },
                None => Param::None,
            };
            commands.push(Cmd { instruction: Instruction::from_string(name)?, param, label });
        } else {
            return None;
        }
    }
    Some(ParsedProgram { variables, commands })
}

fn generate_machinecode(parsed: &ParsedProgram) -> Option<CompilerOutput> {
    let mut compiled = vec![];
    for var in parsed.variables.to_owned() {
        compiled.push(var.value.unwrap_or(0));
    }
    let first_instruction = parsed.variables.len();
    for cmd in parsed.commands.to_owned() {
        let command = match cmd.param {
            Param::Fixed(value) => Command{instruction: cmd.instruction, value},
            // TODO: Checking if this is valid and eventually throw an error.
            Param::None => Command { instruction: cmd.instruction, value: 0 },
            Param::Reference(name) => {
                let resolved_var = resolve_variable(&parsed.variables, &name);
                if resolved_var.is_none() && (cmd.instruction == Instruction::JMP || cmd.instruction == Instruction::JMN) {
                    // TODO: Forbid variable referencing in jumps
                    let resolved_label = resolve_label(&parsed.commands, &name)? + parsed.variables.len();
                    Command { instruction: cmd.instruction, value: resolved_label}
                } else if resolved_var.is_some() {
                    Command{ instruction: cmd.instruction, value: resolved_var?}
                } else {
                    return None;
                }
            } 
        };
        compiled.push(command.to_usize());
    }
    Some(CompilerOutput { mima_code: compiled, start_adress: first_instruction })
}

/*
 * This function resolves variable references. The variables are placed in the first n adresses, so
 * the resolving is simply determining the position in the var array.
 */
fn resolve_variable(variables: &Vec<Variable>, reference: &str) -> Option<usize> {
    let mut memory_position = 0 as usize;
    for var in variables {
        if var.name == *reference {
            return Some(memory_position);
        }
        memory_position+=1;
    }
    return None;
}

fn resolve_label(commands: &Vec<Cmd>, label: &str) -> Option<usize> {
    let mut memory_position = 0 as usize;
    for cmd in commands {
        if cmd.label.is_some() && cmd.label.to_owned().unwrap() == label{
            return Some(memory_position);
        }
        memory_position+=1;
    }
    return None;
}

#[derive(Clone, Debug)]
struct Variable {
    pub name: String,
    pub value: Option<usize>,
}

#[derive(Clone, Debug)]
struct Cmd {
    pub instruction: Instruction,
    pub param: Param,
    pub label: Option<String>,
}

#[derive(Clone, Debug)]
enum Param {
    Fixed(usize),
    Reference(String),
    None,
}

#[cfg(test)]
mod tests {
    use crate::{compiler::compile, mima::Command};

    #[test]
    // Tests a simple addition program
    fn simple_compilation() {
        let assembly_source = 
            "; Add two numbers to a third address

LDV a
ADD b
STV c
HALT

a: DS 22
b: DS 20
c: DS";
        let ldv = Command {instruction: crate::mima::Instruction::LDV, value: 0};
        let add = Command {instruction: crate::mima::Instruction::ADD, value: 1};
        let stv = Command {instruction: crate::mima::Instruction::STV, value: 2};
        let halt = Command {instruction: crate::mima::Instruction::HALT, value: 0};
        let mima_code = vec![22, 20, 0, ldv.to_usize(), add.to_usize(), stv.to_usize(), halt.to_usize()];
        let compiled = compile(assembly_source);
        assert_eq!(compiled.is_some(), true);
        let compiled = compiled.unwrap();
        assert_eq!(compiled.get_mima_code(), mima_code);
        assert_eq!(compiled.get_start_adress(), 3);
    }
    #[test]
    // Test if a code with labels compiles sucessfull
    fn label_compilation() {
        // A simple loop program that counts to 100
        let assembly_source = "one: DS 1
max: DS 100
counter: DS
START: LDV one
STV counter
LOOP: LDV counter
ADD one
STV counter
LDV max
EQL counter
JMN FINISH
JMP LOOP
FINISH: HALT
";
        let mima_code = vec![1, 100, 0, Command{instruction: crate::mima::Instruction::LDV, value: 0}.to_usize(), Command{instruction: crate::mima::Instruction::STV, value: 2}.to_usize(), Command{instruction: crate::mima::Instruction::LDV, value: 2}.to_usize(), Command{instruction: crate::mima::Instruction::ADD, value: 0}.to_usize(), Command{instruction: crate::mima::Instruction::STV, value: 2}.to_usize(), Command{instruction: crate::mima::Instruction::LDV, value: 1}.to_usize(), Command{instruction: crate::mima::Instruction::EQL, value: 2}.to_usize(), Command{instruction: crate::mima::Instruction::JMN, value: 12}.to_usize(), Command{instruction: crate::mima::Instruction::JMP, value: 5}.to_usize(), Command{instruction: crate::mima::Instruction::HALT, value: 0}.to_usize()];
        let compiled = compile(assembly_source);
        assert_eq!(compiled.is_some(), true);
        let compiled = compiled.unwrap();
        assert_eq!(compiled.get_mima_code(), mima_code);
        assert_eq!(compiled.get_start_adress(), 3);
    }
}
