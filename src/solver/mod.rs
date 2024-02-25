pub mod branching;
mod clause_learning;
pub mod config;
pub mod heuristic;
mod literal_watching;
pub mod state;
pub mod statistics;
mod unit_propagation;

use crate::cnf::{Clause, Solution, VarId};
use crate::preprocessor::Preprocessor;
use crate::solver::branching::{AssignmentReason, Brancher};
use crate::solver::clause_learning::ClauseLearner;
use crate::solver::config::Config;
use crate::solver::heuristic::HeuristicType;
use crate::solver::state::State;
use crate::solver::statistics::StateStatistics;
use crate::solver::unit_propagation::UnitPropagator;
use std::collections::HashMap;

pub struct Solver {
    config: Config,
    cnf: Vec<Clause>,
    state: State,
    preprocessor: Preprocessor,
    clause_learner: ClauseLearner,
}

impl Solver {
    pub fn new(clauses: Vec<Clause>, heuristic_type: HeuristicType) -> Self {
        let cnf = clauses;
        let config = Config::new(heuristic_type);
        let preprocessor = Preprocessor::default();
        let clause_learner = ClauseLearner::default();

        Solver {
            config,
            cnf,
            state: State::init(vec![]),
            preprocessor,
            clause_learner,
        }
    }

    pub fn solve(&mut self) -> Solution {
        self.state.stats.start_timing();
        if let Some(solution) = self.preprocess() {
            return solution;
        }
        let mut heuristic = self.config.heuristic.create(&self.state);
        let mut unit_propagator = UnitPropagator::default();
        let mut brancher = Brancher::default();

        loop {
            unit_propagator.propagate(&mut self.state, &mut brancher);

            if let Some(conflict_clause_id) = self.state.conflict_clause_id.clone() {
                // find conflict clause
                let new_clause = self.clause_learner.learn_clause(
                    &mut brancher,
                    &self.state.clauses,
                    conflict_clause_id,
                );

                let new_clause_id = self.state.add_clause(new_clause);

                heuristic.replay_unassignments(brancher.assignments_to_undo());
                let redone_assignment =
                    brancher.backtrack(&mut self.state, &mut unit_propagator, conflict_clause_id);
                if redone_assignment.is_none() {
                    break;
                }
                continue;
            }
            if self.state.is_satisfied() {
                self.state.stats.stop_timing();
                return Some(self.get_solution());
            }

            let next_literal = heuristic.next(&self.state.vars);
            brancher.branch(
                &mut self.state,
                &mut unit_propagator,
                next_literal,
                AssignmentReason::Heuristic,
            );
        }
        self.state.stats.stop_timing();
        None
    }

    /// Preprocesses the clauses and updates the state
    /// Returns SAT, UNSAT or Nothing
    fn preprocess(&mut self) -> Option<Solution> {
        let new_clauses = self.preprocessor.process(self.cnf.clone());
        if new_clauses.is_none() {
            return Some(None);
        }
        self.cnf = new_clauses.unwrap();
        if self.cnf.is_empty() {
            return Some(Some(self.preprocessor.map_solution(HashMap::new())));
        }
        self.state = State::init(self.cnf.clone());
        None
    }

    fn get_solution(&self) -> HashMap<VarId, bool> {
        self.preprocessor.map_solution(self.state.get_assignment())
    }

    pub fn stats(&self) -> &StateStatistics {
        &self.state.stats
    }
}
