pub mod branching;
mod config;
mod heuristic;
pub mod state;
pub mod statistics;
mod unit_propagation;
use crate::cnf::{Clause, VarId};
use crate::preprocessor::Preprocessor;
use crate::solver::branching::Brancher;
use crate::solver::config::Config;
use crate::solver::state::State;
use crate::solver::statistics::StateStatistics;
use crate::solver::unit_propagation::UnitPropagator;
use std::collections::HashMap;

pub struct Solver {
    config: Config,
    state: State,
    preprocessor: Preprocessor,
}

impl Solver {
    pub fn new(clauses: Vec<Clause>) -> Self {
        let state = State::init(clauses);
        let config = Config::default();
        let preprocessor = Preprocessor::default();

        Solver {
            config,
            state,
            preprocessor,
        }
    }

    pub fn solve(&mut self) -> Option<HashMap<VarId, bool>> {
        self.state.stats.start_timing();
        self.preprocess();
        let mut heuristic = self.config.heuristic.create(&self.state);
        let mut unit_propagator = UnitPropagator::new();
        let mut brancher = Brancher::default();

        loop {
            unit_propagator.propagate(&mut self.state, &mut brancher);

            if self.state.in_conflict {
                heuristic.replay_unassignments(brancher.assignments_to_undo());
                let redone_assignment = brancher.backtrack(&mut self.state);
                if redone_assignment.is_none() {
                    self.state.stats.stop_timing();
                    return None;
                }
                continue;
            } else if self.state.is_satisfied() {
                self.state.stats.stop_timing();
                return Some(self.get_solution());
            }

            let assignment = heuristic.next(&self.state.vars);
            brancher.branch(&mut self.state, assignment);
        }
    }

    fn preprocess(&mut self) {
        let new_clauses = self.preprocessor.process(self.state.clauses.clone());
        self.state = State::init(new_clauses);
    }

    fn get_solution(&self) -> HashMap<VarId, bool> {
        self.preprocessor.map_solution(self.state.get_assignment())
    }

    pub fn stats(&self) -> &StateStatistics {
        &self.state.stats
    }
}
