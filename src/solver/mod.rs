mod clause_learning;
pub mod config;
pub mod heuristic;
mod literal_watching;
mod proof_logger;
pub mod state;
pub mod statistics;
pub mod trail;
mod unit_propagation;

use crate::cnf::{Clause, Solution, VarId};
use crate::preprocessor::Preprocessor;
use crate::solver::clause_learning::ClauseLearner;
use crate::solver::config::Config;
use crate::solver::proof_logger::ProofLogger;
use crate::solver::state::State;
use crate::solver::statistics::StateStatistics;
use crate::solver::trail::{AssignmentReason, Trail};
use crate::solver::unit_propagation::UnitPropagator;
use itertools::Itertools;
use std::collections::HashMap;

pub struct Solver {
    config: Config,
    cnf: Vec<Clause>,
    state: State,
    preprocessor: Preprocessor,
    clause_learner: ClauseLearner,
    proof_logger: ProofLogger,
}

impl Solver {
    pub fn new(clauses: Vec<Clause>, config: Config) -> Self {
        let cnf = clauses;
        let preprocessor = Preprocessor::default();
        let clause_learner = ClauseLearner::default();

        Solver {
            cnf,
            state: State::init(vec![]),
            preprocessor,
            clause_learner,
            proof_logger: ProofLogger::new(config.proof_file.is_some()),
            config,
        }
    }

    pub fn solve(&mut self) -> Solution {
        self.state.stats.start_timing();

        if self.config.proof_file.is_none() {
            if let Some(solution) = self.preprocess() {
                return solution;
            }
        }

        if self.is_trivially_unsat() {
            return None;
        }

        // The CNF could have been modified by the preprocessor
        self.state = State::init(self.cnf.clone());

        let mut heuristic = self.config.heuristic.create(&self.state);
        let mut unit_propagator = UnitPropagator::default();
        let mut trail = Trail::default();

        self.enqueue_initial_units(&mut unit_propagator);

        loop {
            unit_propagator.propagate(&mut self.state, &mut trail);

            if let Some(conflict_clause_id) = self.state.conflict_clause_id {
                if trail.decision_level == 0 {
                    break;
                }
                // find conflict clause
                let (new_clause, assertion_level) = self.clause_learner.analyse_conflict(
                    &mut trail,
                    &self.state.clauses,
                    conflict_clause_id,
                );

                self.proof_logger.log(&new_clause);
                let new_clause_id = self.state.add_clause(new_clause);

                heuristic.replay_unassignments(trail.assignments_to_undo(assertion_level));
                trail.backtrack(
                    &mut self.state,
                    &mut unit_propagator,
                    new_clause_id,
                    assertion_level,
                );
                continue;
            }

            if self.state.is_satisfied() {
                self.state.stats.stop_timing();
                return Some(self.get_solution());
            }

            let next_literal = heuristic.next(&self.state.vars);
            trail.assign(
                &mut self.state,
                &mut unit_propagator,
                next_literal,
                AssignmentReason::Heuristic,
            );
        }
        self.state.stats.stop_timing();
        if let Some(proof_file) = self.config.proof_file.as_ref() {
            self.proof_logger.write_to_file(proof_file);
        }

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
        None
    }

    fn is_trivially_unsat(&self) -> bool {
        // contains empty clause
        if self.cnf.iter().any(|clause| clause.literals.is_empty()) {
            return true;
        }

        // contains a unit clause and its negation
        let units = self
            .cnf
            .iter()
            .filter(|clause| clause.literals.len() == 1)
            .map(|clause| clause.literals[0]);
        units.clone().count() != units.clone().map(|x| x.id()).unique().count()
    }

    fn enqueue_initial_units(&self, unit_propagator: &mut UnitPropagator) {
        self.cnf
            .iter()
            .enumerate()
            .filter(|(_, clause)| clause.literals.len() == 1)
            .for_each(|(clause_id, clause)| {
                unit_propagator.enqueue(clause.literals[0], clause_id);
            })
    }

    fn get_solution(&self) -> HashMap<VarId, bool> {
        if self.config.proof_file.is_none() {
            self.preprocessor.map_solution(self.state.get_assignment())
        } else {
            // No need for backmapping if proof is enabled (and preprocessing disabled)
            self.state.get_assignment()
        }
    }

    pub fn stats(&self) -> &StateStatistics {
        &self.state.stats
    }
}
