pub mod basic;
pub mod decay;

use crate::solver::branching::Assignment;
use crate::solver::state::State;

pub trait Heuristic {
    fn init(state: &State) -> Self
    where
        Self: Sized;
    fn replay_unassignments(&mut self, assignments: &[Assignment]);
    fn next(&mut self, vars: &[Option<bool>]) -> Assignment;
}

pub enum HeuristicType {
    TrueFirst,
    Decay,
}

impl HeuristicType {
    pub fn create(&self, state: &State) -> Box<dyn Heuristic> {
        match self {
            HeuristicType::TrueFirst => Box::new(basic::HeuristicTrue::init(state)),
            HeuristicType::Decay => Box::new(decay::HeuristicDecay::init(state)),
        }
    }
}
