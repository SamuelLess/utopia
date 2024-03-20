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
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

pub struct Solver {
    config: Config,
    state: State,
    clause_learner: ClauseLearner,
}

impl Solver {
    pub fn new(clauses: &Vec<Clause>, n_vars: usize, config: Config) -> Self {
        let clause_learner = ClauseLearner::default();

        Solver {
            state: State::init(clauses.clone(), n_vars, config.proof_file.is_some()),
            clause_learner,
            config,
        }
    }

    pub fn solve(&mut self) -> Solution {
        self.state.stats.start_timing();

        if self.is_trivially_unsat() {
            return None;
        }

        let mut heuristic = self.config.heuristic.create(&self.state);
        let mut restarter = Restarter::init(self.config.restart_policy);
        let mut unit_propagator = UnitPropagator::default();
        let mut trail = Trail::new(self.state.num_vars);
        let mut inprocessor = Inprocessor::init(
            &self
                .state
                .clause_database
                .necessary_clauses_iter()
                .map(|clause_id| self.state.clause_database[clause_id].clone())
                .collect_vec(),
        );

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
            } else if self.state.check_satisfied_and_update_blocking_literals() {
                self.state.stats.stop_timing();
                return Some(self.get_solution(&mut inprocessor));
            } else if restarter.check_if_restart_necessary() {
                self.state.stats.num_restarts += 1;
                trail.restart(&mut self.state, heuristic.as_mut());
                if self.config.inprocessing {
                    inprocessor.inprocess(
                        &mut unit_propagator,
                        heuristic.as_mut(),
                        &mut self.state,
                        &mut trail,
                    );
                }
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
            self.state.clause_database.proof_logger.write_to_file(proof_file);
        }

        None
    }

    fn is_trivially_unsat(&self) -> bool {
        // contains empty clause
        if self
            .state
            .clause_database
            .necessary_clauses_iter()
            .map(|clause_id| &self.state.clause_database[clause_id])
            .any(|clause| clause.literals.is_empty())
        {
            return true;
        }

        let cnf = self.state.clause_database.necessary_clauses_iter();
        // contains a unit clause and its negation
        let units = cnf
            .map(|clause_id| &self.state.clause_database[clause_id])
            .filter(|clause| clause.literals.len() == 1)
            .map(|clause| clause.literals[0])
            .collect_vec();

        let positives: HashSet<VarId> = units
            .iter()
            .filter(|lit| lit.positive())
            .map(|lit| lit.id())
            .collect();
        let negatives: HashSet<VarId> = units
            .iter()
            .filter(|lit| !lit.positive())
            .map(|lit| lit.id())
            .collect();

        positives.intersection(&negatives).count() > 0
    }

    fn enqueue_initial_units(&self, unit_propagator: &mut UnitPropagator) {
        self.state
            .clause_database
            .necessary_clauses_iter()
            .enumerate()
            .filter(|(_, clause)| self.state.clause_database[*clause].literals.len() == 1)
            .for_each(|(clause_id, clause)| {
                unit_propagator.enqueue(self.state.clause_database[clause].literals[0], clause_id);
            })
    }

    fn get_solution(&self, inprocessor: &mut Inprocessor) -> HashMap<VarId, bool> {
        let mut assignment = self.state.get_assignment();
        for var in 1..=self.state.num_vars {
            if !assignment.contains_key(&var) {
                assignment.insert(var, true);
            }
        }
        if self.config.inprocessing {
            inprocessor.reconstruct_solution(&mut assignment);
        }
        assignment
    }

    pub fn stats(&self) -> &StateStatistics {
        &self.state.stats
    }
}
