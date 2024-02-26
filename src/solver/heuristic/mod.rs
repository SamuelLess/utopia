pub mod basic;
pub mod decay;

use crate::cnf::Literal;
use crate::solver::state::State;
use crate::solver::trail::Assignment;
use clap::ValueEnum;

pub trait Heuristic {
    fn init(state: &State) -> Self
    where
        Self: Sized;
    fn replay_unassignments(&mut self, assignments: &[Assignment]);
    fn next(&mut self, vars: &[Option<bool>]) -> Literal;
}

#[derive(Debug, Clone, ValueEnum)]
#[clap(rename_all = "kebab_case")]
pub enum HeuristicType {
    #[clap(name = "decay")]
    Decay,
    #[clap(name = "true-first")]
    TrueFirst,
}

impl HeuristicType {
    pub fn create(&self, state: &State) -> Box<dyn Heuristic> {
        match self {
            HeuristicType::Decay => Box::new(decay::HeuristicDecay::init(state)),
            HeuristicType::TrueFirst => Box::new(basic::HeuristicTrue::init(state)),
        }
    }
}
