pub mod basic;
pub mod decay;
mod vmtf;
mod vsids;

use crate::cnf::{Clause, Literal};
use crate::solver::state::State;
use crate::solver::trail::Assignment;
use clap::ValueEnum;

pub trait Heuristic {
    fn init(state: &State) -> Self
    where
        Self: Sized;
    fn unassign(&mut self, assignments: &Assignment);

    fn conflict(&mut self, _clause: &Clause) {
        // by default, do nothing
    }

    fn next(&mut self, vars: &[Option<bool>]) -> Literal;
}

#[derive(Debug, Clone, ValueEnum)]
pub enum HeuristicType {
    #[clap(name = "decay")]
    Decay,
    #[clap(name = "true-first")]
    TrueFirst,
    #[clap(name = "vmtf")]
    VMTF,
    #[clap(name = "vsids")]
    VSIDS,
}

impl HeuristicType {
    pub fn create(&self, state: &State) -> Box<dyn Heuristic> {
        match self {
            HeuristicType::Decay => Box::new(decay::HeuristicDecay::init(state)),
            HeuristicType::TrueFirst => Box::new(basic::HeuristicTrue::init(state)),
            HeuristicType::VMTF => Box::new(vmtf::HeuristicVMTF::init(state)),
            HeuristicType::VSIDS => Box::new(vsids::HeuristicVSIDS::init(state)),
        }
    }
}
