use dynamic_tournament_generator::{
    options::TournamentOptions, EntrantScore, EntrantSpot, Entrants, Match, Matches, Node,
};

use crate::{instructions::Instructions, VirtualMachine};

#[derive(Clone, Debug)]
pub struct Tournament<T> {
    creation: Instructions,
    update: Instructions,
    render: Instructions,
    entrants: Entrants<T>,
    matches: Matches<EntrantScore<u64>>,
    options: TournamentOptions,
}

impl<T> Tournament<T> {
    pub fn new<I>(instructions: I, options: TournamentOptions)
    where
        I: Into<Instructions>,
    {
    }

    pub fn push(&mut self, entrant: T) {}
}

impl<T> Extend<T> for Tournament<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {}
}

/// The virtual machine used for tournament creation.
pub struct CreationMachine {
    vm: VirtualMachine,
}

impl CreationMachine {
    pub fn new(instructions: Instructions) -> Self {
        Self {
            vm: VirtualMachine::new(instructions),
        }
    }

    pub fn run<T>(&mut self, entrants: &[T]) -> Matches<EntrantScore<u64>> {
        self.vm.clear();
        let mut stack = self.vm.stack_mut();
        stack.push(entrants.len() as u64);
        for _ in 0..entrants.len() {
            stack.push(0);
        }

        let start = self.vm.rax();
        let stack = self.vm.stack();
        let len = stack[start as usize];

        let mut matches = Matches::with_capacity(len as usize);

        let mut index = 0;
        while index < (len as usize) {
            let s0 = match stack[index] {
                u64::MAX => EntrantSpot::Empty,
                n if n == u64::MAX - 1 => EntrantSpot::TBD,
                n => EntrantSpot::Entrant(Node::new(n as usize)),
            };

            let s1 = match stack[index + 1] {
                u64::MAX => EntrantSpot::Empty,
                n if n == u64::MAX - 1 => EntrantSpot::TBD,
                n => EntrantSpot::Entrant(Node::new(n as usize)),
            };

            matches.push(Match::new([s0, s1]));
            index += 2;
        }

        matches
    }
}

pub struct UpdateMachine {
    vm: VirtualMachine,
}

#[cfg(test)]
mod tests {
    use crate::instructions::{Instruction, Instructions, Location, Operand};

    use super::CreationMachine;

    #[test]
    fn test_single_elimination_creation() {
        let instructions = vec![Instruction::JGE(
            Operand::pointer(0),
            Operand::Const(3),
            Location(8),
        )];

        let mut vm = CreationMachine::new();
    }
}
