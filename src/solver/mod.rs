mod clause_database;
mod clause_learning;
pub mod config;
mod ema_policy;
pub mod heuristic;
mod inprocessor;
mod literal_watching;
mod proof_logger;
pub mod restarts;
pub mod state;
pub mod statistics;
pub mod trail;
mod unit_propagation;

use crate::cnf::{Clause, Literal, Solution, VarId};
use crate::solver::clause_learning::ClauseLearner;
use crate::solver::config::Config;
use crate::solver::inprocessor::Inprocessor;
use crate::solver::proof_logger::ProofLogger;
use crate::solver::restarts::Restarter;
use crate::solver::state::State;
use crate::solver::statistics::StateStatistics;
use crate::solver::trail::{AssignmentReason, Trail};
use crate::solver::unit_propagation::UnitPropagator;
use std::collections::{HashMap, HashSet};

pub struct Solver {
    config: Config,
    state: State,
    cnf: Vec<Clause>,
    clause_learner: ClauseLearner,
    proof_logger: ProofLogger,
}

impl Solver {
    pub fn new(clauses: Vec<Clause>, config: Config) -> Self {
        let clause_learner = ClauseLearner::default();

        Solver {
            cnf: clauses.clone(),
            state: State::init(clauses.clone()),
            clause_learner,
            proof_logger: ProofLogger::new(config.proof_file.is_some()),
            config,
        }
    }

    pub fn solve(&mut self) -> Solution {
        self.state.stats.start_timing();

        if self.is_trivially_unsat() {
            return None;
        }

        // The CNF could have been modified by the preprocessor
        self.state = State::init(self.cnf.clone());

        let mut heuristic = self.config.heuristic.create(&self.state);
        let mut restarter = Restarter::init(self.config.restart_policy);
        let mut unit_propagator = UnitPropagator::default();
        let mut trail = Trail::new(self.state.num_vars);
        let mut inprocessor = Inprocessor::init();

        self.enqueue_initial_units(&mut unit_propagator);

        loop {
            unit_propagator.propagate(&mut self.state, &mut trail);

            if let Some(conflict_clause_id) = self.state.conflict_clause_id {
                if trail.decision_level == 0 {
                    break;
                }
                self.state
                    .clause_database
                    .delete_clauses_if_necessary(&mut self.state.literal_watcher, &trail);

                // find conflict clause
                let (new_clause, assertion_level) = self.clause_learner.analyse_conflict(
                    &mut trail,
                    &self.state.clause_database,
                    conflict_clause_id,
                );

                self.proof_logger.log(&new_clause);
                restarter.conflict(new_clause.lbd.unwrap(), trail.assignment_stack.len());

                // The first literal is always UIP
                let uip = new_clause.literals[0];
                let new_clause_id = self
                    .state
                    .clause_database
                    .add_clause(new_clause, &mut self.state.literal_watcher);

                unit_propagator.enqueue(uip, new_clause_id);

                heuristic.conflict(&self.state.clause_database[conflict_clause_id]);
                trail.backtrack(&mut self.state, heuristic.as_mut(), assertion_level);
                inprocessor.inprocess(
                    &mut self.state.clause_database,
                    &mut self.state.literal_watcher,
                    &trail,
                );
            } else if self.state.is_satisfied() {
                self.state.stats.stop_timing();
                return Some(self.get_solution());
            } else if restarter.check_if_restart_necessary() {
                self.state.stats.num_restarts += 1;
                trail.restart(&mut self.state, heuristic.as_mut());
            } else {
                let next_var = heuristic.next(&self.state.vars);
                let next_literal = Literal::from_value(next_var, self.state.var_phases[next_var]);
                trail.assign(
                    &mut self.state,
                    &mut unit_propagator,
                    next_literal,
                    AssignmentReason::Heuristic,
                );
            }
        }
        self.state.stats.stop_timing();
        if let Some(proof_file) = self.config.proof_file.as_ref() {
            self.proof_logger.write_to_file(proof_file);
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
        let positives: HashSet<VarId> = units
            .clone()
            .filter(|lit| lit.positive())
            .map(|lit| lit.id())
            .collect();
        let negatives: HashSet<VarId> = units
            .clone()
            .filter(|lit| !lit.positive())
            .map(|lit| lit.id())
            .collect();

        positives.intersection(&negatives).count() > 0
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
        self.state.get_assignment()
    }

    pub fn stats(&self) -> &StateStatistics {
        &self.state.stats
    }
}
