use std::ops::Deref;

#[derive(Clone, Debug)]
pub enum Instruction {
    ADD(Operand),
    SUB(Operand),
    MUL(Operand),
    DIV(Operand),
    SHL(Pointer),
    SHR(Pointer),

    AND(Operand),
    OR(Operand),
    XOR(Operand),
    NOT,

    MOV(Operand, Operand),
    PUSH,
    POP,

    JMP(Location),
    JE(Operand, Operand, Location),
    JNE(Operand, Operand, Location),
    JG(Operand, Operand, Location),
    JGE(Operand, Operand, Location),
    JL(Operand, Operand, Location),
    JLE(Operand, Operand, Location),

    ABORT,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Operand {
    Const(u64),
    Pointer(Pointer),
}

impl Operand {
    pub fn pointer(val: u64) -> Self {
        Self::Pointer(Pointer(val))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Pointer(pub u64);

impl Deref for Pointer {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Location(pub u64);

#[derive(Clone, Debug)]
pub struct Instructions(pub(crate) Vec<Instruction>);

impl From<Vec<Instruction>> for Instructions {
    fn from(instructions: Vec<Instruction>) -> Self {
        Self(instructions)
    }
}
