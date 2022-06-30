use instructions::{Instruction, Instructions, Operand};

mod instructions;

#[derive(Clone, Debug)]
pub struct VirtualMachine {
    instructions: Instructions,
    location: u64,
    reg: u64,
    stack: Vec<u64>,
}

impl VirtualMachine {
    pub fn new<T>(instructions: T) -> Self
    where
        T: Into<Instructions>,
    {
        Self {
            instructions: instructions.into(),
            location: 0,
            reg: 0,
            stack: Vec::new(),
        }
    }

    pub fn peek(&mut self) -> Option<()> {
        let instruction = &self.instructions.0[self.location as usize];

        match instruction {
            Instruction::ADD(rhs) => self.reg += self.get(*rhs)?,
            Instruction::SUB(rhs) => self.reg -= self.get(*rhs)?,
            Instruction::MUL(rhs) => self.reg *= self.get(*rhs)?,
            Instruction::DIV(rhs) => self.reg /= self.get(*rhs)?,
            _ => return None,
        }

        self.location += 1;
        None
    }

    pub fn run(&mut self) -> Option<()> {
        while (self.location as usize) < self.instructions.0.len() {
            self.peek()?;
        }

        None
    }

    pub fn complete(mut self) -> Option<(u64, Vec<u64>)> {
        self.run();
        Some((self.reg, self.stack))
    }

    /// Adds `rhs` to `%reg`.
    #[inline]
    fn add(&mut self, rhs: Operand) -> Option<()> {
        self.reg += self.get(rhs)?;
        Some(())
    }

    #[inline]
    fn sub(&mut self, rhs: Operand) -> Option<()> {
        self.reg -= self.get(rhs)?;
        Some(())
    }

    #[inline]
    fn mul(&mut self, rhs: Operand) -> Option<()> {
        self.reg *= self.get(rhs)?;
        Some(())
    }

    #[inline]
    fn div(&mut self, rhs: Operand) -> Option<()> {
        self.reg /= self.get(rhs)?;
        Some(())
    }

    fn and(&mut self, rhs: Operand) -> Option<()> {
        self.reg &= self.get(rhs)?;
        Some(())
    }

    fn or(&mut self, rhs: Operand) -> Option<()> {
        self.reg |= self.get(rhs)?;
        Some(())
    }

    fn not(&mut self) -> Option<()> {
        self.reg = !self.reg;
        Some(())
    }

    fn xor(&mut self, rhs: Operand) -> Option<()> {
        self.reg ^= self.get(rhs)?;
        Some(())
    }

    /// Returns the value for operand.
    fn get(&self, operand: Operand) -> Option<u64> {
        match operand {
            Operand::Const(v) => Some(v),
            Operand::Pointer(ptr) => self.stack.get(*ptr as usize).map(|v| *v),
        }
    }
}

pub struct Error {}

pub enum ErrorKind {
    InvalidAddress,
    DivideByZero,
    IntegerOverflow,
}

#[cfg(test)]
mod tests {
    use super::VirtualMachine;
    use crate::instructions::{Instruction, Instructions, Operand, Pointer};

    #[test]
    fn test_virtual_machine_add() {
        let instructions = vec![
            Instruction::ADD(Operand::Const(50)),
            Instruction::ADD(Operand::Pointer(Pointer(0))),
        ];

        let mut vm = VirtualMachine::new(instructions);
        vm.reg = 210;
        vm.stack.push(500);

        vm.peek();
        assert_eq!(vm.reg, 260);

        vm.run();
        assert_eq!(vm.reg, 760);
    }

    #[test]
    fn test_virtual_machine_sub() {
        let instructions = vec![
            Instruction::SUB(Operand::Const(60)),
            Instruction::SUB(Operand::Pointer(Pointer(0))),
        ];

        let mut vm = VirtualMachine::new(instructions);
        vm.reg = 70;
        vm.stack.push(9);

        vm.peek();
        assert_eq!(vm.reg, 10);

        vm.peek();
        assert_eq!(vm.reg, 1);
    }

    #[test]
    fn test_virtual_machine_mul() {
        let instructions = vec![
            Instruction::MUL(Operand::Const(5)),
            Instruction::MUL(Operand::Pointer(Pointer(0))),
        ];

        let mut vm = VirtualMachine::new(instructions);
        vm.reg = 6;
        vm.stack.push(5);

        vm.peek();
        assert_eq!(vm.reg, 5 * 6);

        vm.peek();
        assert_eq!(vm.reg, 5 * 6 * 5);
    }

    #[test]
    fn test_virtual_machine_div() {
        let instructions = vec![
            Instruction::DIV(Operand::Const(10)),
            Instruction::DIV(Operand::Pointer(Pointer(0))),
        ];

        let mut vm = VirtualMachine::new(instructions);
        vm.reg = 100;
        vm.stack.push(5);

        vm.peek();
        assert_eq!(vm.reg, 100 / 10);

        vm.peek();
        assert_eq!(vm.reg, 100 / 10 / 5);
    }
}
