use crate::cnf::{Clause, ClauseId, Literal, VarId};
use crate::solver::heuristic::Heuristic;
use crate::solver::state::State;
use crate::solver::trail::AssignmentReason;
use crate::solver::trail::Trail;
use crate::solver::unit_propagation::UnitPropagator;
use itertools::Itertools;
use std::collections::HashMap;

pub struct Inprocessor {
    conflict_count: usize,
    bve_reconstruction_data: Vec<(Literal, Clause)>,
    start_time: std::time::Instant,
    total_time: std::time::Duration,
}

impl Inprocessor {
    pub fn init() -> Self {
        Inprocessor {
            conflict_count: 10000, /*TODO: conflict_count := 0*/
            bve_reconstruction_data: vec![],
            start_time: std::time::Instant::now(),
            total_time: std::time::Duration::from_secs(0),
        }
    }

    pub fn inprocess(
        &mut self,
        unit_propagator: &mut UnitPropagator,
        heuristic: &mut dyn Heuristic,
        state: &mut State,
        trail: &mut Trail,
    ) {
        return;
        // remove all unit-assignments from the trail. This makes adding arbitrary clauses much
        // easier, as we can re-initalize the trail with the new clauses.

        let units = trail
            .assignment_stack
            .iter()
            .map(|x| {
                // check preconditions for inprocessing
                if let AssignmentReason::Forced(clause_id) = x.reason {
                    if x.decision_level != 0 {
                        panic!("Inprocessing called at decision level != 0");
                    }
                    (x.literal, clause_id)
                } else {
                    panic!("Inprocessing called at decision level != 0");
                }
            })
            .collect::<Vec<_>>();
        trail.backtrack_completely(state, heuristic);

        for (lit, _) in units.iter() {
            state.unassign(*lit);
        }

        println!("Inprocessing");

        let vars = state
            .clause_database
            .iter()
            .flat_map(|clause| state.clause_database[clause].literals.iter())
            .map(|literal| literal.id())
            .unique()
            .collect_vec();

        for var in vars {
            // don't look at this code -- serious risk of brain damage :D
            let occ = state
                .clause_database
                .iter()
                .flat_map(|clause_id| {
                    state.clause_database[clause_id]
                        .clone()
                        .map(move |x| (x, clause_id))
                })
                .into_group_map();

            self.bounded_variable_elimination(var, trail, unit_propagator, state, &occ);
        }

        println!("CNF After:");
        // enqueue all units again
        for (unit_literal, conflict_clause_id) in units {
            unit_propagator.enqueue(unit_literal, conflict_clause_id);
        }
    }

    /// BVE based on "Inprocessing Rules" by Matti Järvisalo, Marijn Heule, Armin Biere
    fn bounded_variable_elimination(
        &mut self,
        var_id: VarId,
        trail: &Trail,
        unit_propagator: &mut UnitPropagator,
        state: &mut State,
        occ: &HashMap<Literal, Vec<ClauseId>>,
    ) {
        let mut resolution_clauses = vec![];

        let empty_vec = Vec::new();

        // group occ by 1. pos/neg occ 2. occ in learned/non-learned clauses
        let group_occ = |sign| {
            occ.get(&Literal::from_value(var_id, sign))
                .unwrap_or(&empty_vec)
        };

        // find all pos_occ and neg_occ
        let pos_occ = group_occ(true);
        let neg_occ = group_occ(false);

        let num_clauses_before = pos_occ.len() + neg_occ.len();

        // do resolution with the non-learned clauses
        let pairs = pos_occ
            .iter()
            .filter(|clause_id| state.clause_database[**clause_id].lbd.is_none())
            .cartesian_product(
                neg_occ
                    .iter()
                    .filter(|clause_id| state.clause_database[**clause_id].lbd.is_none()),
            );

        for (clause_1, clause_2) in pairs {
            let c1_iter = state.clause_database[*clause_1].literals.iter();
            let c2_iter = state.clause_database[*clause_2].literals.iter();

            let resolution_clause = c1_iter.chain(c2_iter).filter(|lit| lit.id() != var_id);

            // deduplicate new_clause
            let unique = resolution_clause.unique().collect_vec();

            // check for tautology
            if unique.len() == unique.iter().map(|lit| lit.id()).unique().count() {
                resolution_clauses.push(Clause::from(unique.iter().map(|lit| **lit).collect_vec()));
            }

            if resolution_clauses.len() >= num_clauses_before {
                return; // This won't be worthwhile. Abort and don't execute resolution.
            }
        }

        // delete old clauses
        for (any_occ, polarity_in_clause) in [(pos_occ, true), (neg_occ, false)] {
            for clause_id in any_occ.iter() {
                // if clause is required then add to bve_reconstruction_data
                if state.clause_database[*clause_id].lbd.is_none() {
                    self.bve_reconstruction_data.push((
                        Literal::from_value(var_id, polarity_in_clause),
                        state.clause_database[*clause_id].clone(),
                    ));
                }

                state.clause_database.delete_clause_if_allowed(
                    *clause_id,
                    &mut state.literal_watcher,
                    trail,
                );
            }
        }

        // add clauses as required clauses
        for clause in &resolution_clauses {
            let clause_id = state
                .clause_database
                .add_clause(clause.clone(), &mut state.literal_watcher);

            // newly found units have to be enqueued
            if clause.literals.len() == 1 {
                unit_propagator.enqueue(clause.literals[0], clause_id);
            }
        }

        let num_added_clauses = resolution_clauses.len();

        assert!(num_added_clauses <= num_clauses_before);

        println!(
            "Resolved {num_clauses_before} clauses for {}",
            num_added_clauses
        );
    }

    pub fn stop_timing(&mut self) {
        self.total_time += self.start_time.elapsed();
    }

    pub fn start_timing(&mut self) {
        self.start_time = std::time::Instant::now();
    }
}
