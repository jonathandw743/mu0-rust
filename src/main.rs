use std::{collections::HashMap, fs, vec};

pub struct MU0 {
    pub acc: i16,
    pub pc: u16,
    pub ir: u16,

    pub n: bool,
    pub z: bool,

    pub mem: [u16; 4096],
}
impl MU0 {
    pub fn new() -> Self {
        Self {
            acc: 0,
            pc: 0,
            ir: 0,

            n: false,
            z: true,

            mem: [0; 4096],
        }
    }
    pub fn load_program(&mut self, program: Vec<u16>) {
        for (i, word) in program.iter().enumerate() {
            self.mem[i as usize] = *word;
        }
    }
    pub fn mem_dump(&self) -> String {
        self.mem
            .iter()
            .map(|x| format!("{:016b}", *x))
            .reduce(|a, b| format!("{}\n{}", a, b))
            .unwrap()
    }
    pub fn mem_dump_smart(&self) -> String {
        let mut last_word_address = self.mem.len();
        for word in self.mem.iter().rev() {
            if *word != 0 {
                break;
            }
            last_word_address -= 1;
        }
        (0..last_word_address)
            .into_iter()
            .map(|i| format!("{:016b}", self.mem[i]))
            .reduce(|a, b| format!("{}\n{}", a, b))
            .unwrap()
    }
    pub fn fetch(&mut self) {
        self.ir = self.mem[self.pc as usize];
        self.pc += 1;
        self.pc = self.pc % 4096;
    }
    pub fn execute(&mut self) -> Option<Interrupt> {
        let opcode = self.ir >> 12;
        let operand = self.ir & (u16::MAX >> 4);
        self.instruction(opcode, operand)
    }
    pub fn instruction(&mut self, opcode: u16, operand: u16) -> Option<Interrupt> {
        match opcode {
            0 => self.lda(operand),
            1 => self.sta(operand),
            2 => self.add(operand),
            3 => self.sub(operand),
            4 => self.jmp(operand),
            5 => self.jge(operand),
            6 => self.jne(operand),
            7 => self.stp(),
            8 => self.swi(operand),
            _ => None,
        }
    }
    pub fn set_flags(&mut self) {
        self.z = self.acc == 0;
        self.n = self.acc < 0;
    }
    pub fn lda(&mut self, operand: u16) -> Option<Interrupt> {
        self.acc = self.mem[operand as usize] as i16;
        self.set_flags();
        // println!("lda");
        None
    }
    pub fn sta(&mut self, operand: u16) -> Option<Interrupt> {
        self.mem[operand as usize] = self.acc as u16;
        // println!("sta");
        None
    }
    pub fn add(&mut self, operand: u16) -> Option<Interrupt> {
        self.acc += self.mem[operand as usize] as i16;
        self.set_flags();
        // println!("add");
        None
    }
    pub fn sub(&mut self, operand: u16) -> Option<Interrupt> {
        self.acc -= self.mem[operand as usize] as i16;
        self.set_flags();
        // println!("sub");
        None
    }
    pub fn jmp(&mut self, operand: u16) -> Option<Interrupt> {
        self.pc = operand;
        // println!("jmp");
        None
    }
    pub fn jge(&mut self, operand: u16) -> Option<Interrupt> {
        if !self.n {
            self.pc = operand;
        }
        // println!("jge");
        None
    }
    pub fn jne(&mut self, operand: u16) -> Option<Interrupt> {
        if !self.z {
            self.pc = operand;
        }
        // println!("jne");
        None
    }
    pub fn swi(&self, operand: u16) -> Option<Interrupt> {
        // println!("swi");
        match operand {
            0 => Some(Interrupt::NumOut(self.acc)),
            _ => None,
        }
    }
    pub fn stp(&mut self) -> Option<Interrupt> {
        Some(Interrupt::Halt)
    }
    pub fn assemble(program: Vec<&str>) -> [u16; 4096] {
        let mut program_expressions = Vec::new();
        for word in program {
            let mut expressions = word.trim();
            if let Some(position) = expressions.char_indices().position(|x| x.1 == ';') {
                expressions = expressions.get(0..position).unwrap().trim();
            }
            if expressions.is_empty() {
                continue;
            }
            let expressions = expressions.split(" ");
            let expressions = expressions.map(|x| x.trim());
            let expressions = expressions.filter(|x| !x.is_empty());
            let expressions = expressions.map(|x| x.to_lowercase());
            let expressions: Vec<_> = expressions.collect();

            program_expressions.push(expressions);
        }
        let mut current_address: usize = 0;
        let mut definitions: HashMap<String, u16> = HashMap::new();
        for expressions in &mut program_expressions {
            match expressions[0].as_str() {
                "lda" | "hello" | "sta" | "add" | "sub" | "jmp" | "jge" | "jne" | "stp"
                | "defw" | "swi" => {
                    current_address += 1;
                }
                "org" => {
                    current_address = expressions[1].parse().unwrap();
                }
                _ => match expressions.get(1).map(|x| x.as_str()) {
                    Some("equ") => {
                        definitions.insert(expressions[0].clone(), expressions[2].parse().unwrap());
                        expressions.clear();
                    }
                    _ => {
                        definitions.insert(
                            expressions[0].clone(),
                            (current_address as u16) & (!(0b1111 << 12)),
                        );
                        expressions.remove(0);
                        if !expressions.is_empty() {
                            current_address += 1;
                        }
                    }
                },
            };
        }
        program_expressions.retain(|expressions| !expressions.is_empty());
        let mut current_address: usize = 0;
        let mut result = [0; 4096];
        for expressions in program_expressions {
            match expressions[0].as_str() {
                "org" => {
                    current_address = expressions[1].parse().unwrap();
                }
                "lda" => {
                    result[current_address] =
                        (0b000 << 12) | definitions.get(&expressions[1]).unwrap();
                    current_address += 1;
                }
                "sta" => {
                    result[current_address] =
                        (0b001 << 12) | definitions.get(&expressions[1]).unwrap();
                    current_address += 1;
                }
                "add" => {
                    result[current_address] =
                        (0b010 << 12) | definitions.get(&expressions[1]).unwrap();
                    current_address += 1;
                }
                "sub" => {
                    result[current_address] =
                        (0b011 << 12) | definitions.get(&expressions[1]).unwrap();
                    current_address += 1;
                }
                "jmp" => {
                    result[current_address] =
                        (0b100 << 12) | definitions.get(&expressions[1]).unwrap();
                    current_address += 1;
                }
                "jge" => {
                    result[current_address] =
                        (0b101 << 12) | definitions.get(&expressions[1]).unwrap();
                    current_address += 1;
                }
                "jne" => {
                    result[current_address] =
                        (0b110 << 12) | definitions.get(&expressions[1]).unwrap();
                    current_address += 1;
                }
                "stp" => {
                    result[current_address] = 0b111 << 12;
                    current_address += 1;
                }
                "defw" => {
                    result[current_address] = expressions[1].parse().unwrap();
                    current_address += 1;
                }
                "swi" => {
                    result[current_address] = (0b1000 << 12)
                        | (expressions[1].parse::<u16>().unwrap() & (!(0b1111 << 12)));
                    current_address += 1;
                }
                _ => {
                    panic!("two labels in a row somewhere")
                }
            };
        }
        result
    }
}
pub enum Interrupt {
    Halt,
    NumOut(i16),
}
fn main() {
    let mut mu0 = MU0::new();
    mu0.mem = MU0::assemble(
        fs::read_to_string("src/program.txt")
            .unwrap()
            .split("\n")
            .collect(),
    );
    // println!("{}", mu0.mem_dump_smart());
    // println!("");
    loop {
        mu0.fetch();
        match mu0.execute() {
            Some(Interrupt::Halt) => {
                break;
            }
            Some(Interrupt::NumOut(x)) => {
                println!("{}", x);
            }
            _ => {}
        }
    }
    // println!("");
    // println!("{}", mu0.mem_dump_smart());
}
