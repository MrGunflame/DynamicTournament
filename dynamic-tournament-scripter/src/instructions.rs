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
    NOT(Operand),
    XOR(Operand),

    MOV(Operand, Operand),
    PUSH,
    POP,

    JMP(Location),
    JE(Pointer, Pointer, Location),
    JNE(Pointer, Pointer, Location),
    JG(Pointer, Pointer, Location),
    JGE(Pointer, Pointer, Location),
    JL(Pointer, Pointer, Location),
    JLE(Pointer, Pointer, Location),

    ABORT,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Operand {
    Const(u64),
    Pointer(Pointer),
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
pub struct Location(u64);

#[derive(Clone, Debug)]
pub struct Instructions(pub(crate) Vec<Instruction>);

impl From<Vec<Instruction>> for Instructions {
    fn from(instructions: Vec<Instruction>) -> Self {
        Self(instructions)
    }
}
