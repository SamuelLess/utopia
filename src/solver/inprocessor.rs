use crate::cnf::{Clause, ClauseId, Literal, VarId};
use crate::solver::heuristic::Heuristic;
use crate::solver::state::State;
use crate::solver::trail::AssignmentReason;
use crate::solver::trail::Trail;
use crate::solver::unit_propagation::UnitPropagator;
use itertools::Itertools;
use std::collections::{HashMap, VecDeque};

const INPROCESSING_RATIO: f64 = 0.15;

const DETERMINISTIC: bool = false;
// sat/ii32b4.cnf

pub struct Inprocessor {
    bve_reconstruction_data: Vec<(Literal, Clause)>,
    initialization_time: std::time::Instant,
    total_inprocessing_time: std::time::Duration,
    current_inprocessing_start: std::time::Instant,
    bve_queue: VecDeque<VarId>,
}

impl Inprocessor {
    pub fn init(cnf: &Vec<Clause>) -> Self {
        let start_time = std::time::Instant::now();
        let vars_ordered_by_occurences = cnf
            .iter()
            .flat_map(|clause| clause.literals.iter())
            .map(|literal| literal.id())
            .counts()
            .iter()
            .sorted_by_cached_key(|(var_id, &count)| (count, **var_id))
            .map(|(var, _)| *var)
            .collect::<VecDeque<VarId>>();

        println!(
            "Ordered vars in {} ms",
            start_time.elapsed().as_secs_f64() / 1000.0
        );

        Inprocessor {
            bve_reconstruction_data: vec![],
            initialization_time: std::time::Instant::now(),
            total_inprocessing_time: std::time::Duration::from_secs(0),
            current_inprocessing_start: std::time::Instant::now(),
            bve_queue: vars_ordered_by_occurences,
        }
    }

    pub fn start_inprocessing(
        &mut self,
        trail: &mut Trail,
        state: &mut State,
        heuristic: &mut dyn Heuristic,
    ) -> Vec<(Literal, ClauseId)> {
        self.current_inprocessing_start = std::time::Instant::now();

        assert_eq!(
            trail.decision_level, 0,
            "Inprocessing called at decision level != 0"
        );

        println!("Starting inprocessing");

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

        units
    }

    pub fn end_inprocessing(
        &mut self,
        units: Vec<(Literal, ClauseId)>,
        unit_propagator: &mut UnitPropagator,
    ) {
        // enqueue all units again
        for (unit_literal, conflict_clause_id) in units {
            unit_propagator.enqueue(unit_literal, conflict_clause_id);
        }

        println!(
            "Inprocessing took {} ms",
            self.current_inprocessing_start.elapsed().as_secs_f64() * 1000.0
        );

        self.total_inprocessing_time += self.current_inprocessing_start.elapsed();
    }

    pub fn should_interrupt(&self) -> bool {
        if DETERMINISTIC {
            return true;
        }
        (self.total_inprocessing_time + self.current_inprocessing_start.elapsed()).as_secs_f64()
            > self.initialization_time.elapsed().as_secs_f64() * INPROCESSING_RATIO
    }

    pub fn should_start_inprocessing(&self) -> bool {
        if DETERMINISTIC {
            return true;
        }
        self.total_inprocessing_time.as_secs_f64() + 0.1
            < self.initialization_time.elapsed().as_secs_f64() * INPROCESSING_RATIO
    }

    pub fn inprocess(
        &mut self,
        unit_propagator: &mut UnitPropagator,
        heuristic: &mut dyn Heuristic,
        state: &mut State,
        trail: &mut Trail,
    ) {
        if !self.should_start_inprocessing() {
            return;
        }

        // remove all unit-assignments from the trail. This makes adding arbitrary clauses much
        // easier, as we can re-initalize the trail with the new clauses.
        let units = self.start_inprocessing(trail, state, heuristic);

        while let Some(var) = self.bve_queue.pop_front() {
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

            if self.should_interrupt() {
                break;
            }
        }

        self.end_inprocessing(units, unit_propagator);
    }

    /// Reconstruction as described in M. Järvisalo, M. J. H. Heule, and A. Biere,
    /// “Inprocessing Rules,” in Automated Reasoning, vol. 7364, B. Gramlich, D. Miller,
    /// and U. Sattler, Eds., Berlin, Heidelberg: Springer Berlin Heidelberg, 2012, pp. 355–370.
    /// doi: 10.1007/978-3-642-31365-3_28.
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
                //state.verify_watches();
            }
        }

        // add clauses as required clauses
        for clause in &resolution_clauses {
            let clause_id = state
                .clause_database
                .add_clause(clause.clone(), &mut state.literal_watcher);

            // newly found units have to be enqueued
            if clause.literals.len() == 1 {
                println!("Enqueued newly generated unit");
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

    /// Reconstruction as described in M. Järvisalo, M. J. H. Heule, and A. Biere,
    /// “Inprocessing Rules,” in Automated Reasoning, vol. 7364, B. Gramlich, D. Miller,
    /// and U. Sattler, Eds., Berlin, Heidelberg: Springer Berlin Heidelberg, 2012, pp. 355–370.
    /// doi: 10.1007/978-3-642-31365-3_28.
    pub fn reconstruct_solution(&mut self, solution: &mut HashMap<VarId, bool>) {
        while let Some((literal, clause)) = self.bve_reconstruction_data.pop() {
            let clause_is_sat = clause
                .literals
                .iter()
                .any(|lit| *solution.get(&(lit.id())).unwrap() == (lit.positive()));

            if !clause_is_sat {
                if literal.positive() {
                    solution.insert(literal.id(), true);
                } else {
                    solution.insert(literal.id(), false);
                }
            }
        }
    }
}
