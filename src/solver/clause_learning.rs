use crate::cnf::{Clause, ClauseId, Literal, VarId};
use crate::solver::clause_database::ClauseDatabase;
use crate::solver::trail::{AssignmentReason, Trail};
use itertools::Itertools;
use std::collections::HashSet;

use fnv::FnvHasher;
use std::hash::BuildHasherDefault;

type FastHasher = BuildHasherDefault<FnvHasher>;

#[derive(Debug, Default, Clone)]
pub struct ClauseLearner {}

impl ClauseLearner {
    /// Assumes that the current state is in conflict
    pub fn analyse_conflict(
        &mut self,
        trail: &mut Trail,
        clause_database: &mut ClauseDatabase,
        conflict_clause_id: ClauseId,
    ) -> (Clause, usize) {
        let mut learned_clause = vec![];

        // find learned clause
        let mut count = 0;
        let mut current_literal: Option<Literal> = None;
        let mut current_reason_clause_id: ClauseId = conflict_clause_id;
        let mut trail_position = trail.assignment_stack.len() - 1;
        let mut seen: HashSet<VarId, FastHasher> = HashSet::with_hasher(FastHasher::default());

        loop {
            let conflict_clause = &mut clause_database[current_reason_clause_id];
            conflict_clause.update_lbd(trail);

            for lit in conflict_clause.literals.clone() {
                if current_literal.is_some() && lit.id() == current_literal.unwrap().id() {
                    continue; // current literal is not part of the reason clause
                }

                if !seen.contains(&lit.id()) && trail.var_decision_level[lit.id()] > 0 {
                    seen.insert(lit.id());

                    assert!(trail.var_decision_level[lit.id()] <= trail.decision_level);
                    if trail.var_decision_level[lit.id()] == trail.decision_level {
                        count += 1;
                    } else {
                        learned_clause.push(lit);
                    }
                }
            }

            // find next literal
            while !seen.contains(&trail.assignment_stack[trail_position].literal.id()) {
                trail_position -= 1;
            }
            current_literal = Some(trail.assignment_stack[trail_position].literal);

            seen.remove(&current_literal.unwrap().id());
            count -= 1;
            if count == 0 {
                break;
            }

            current_reason_clause_id = match trail.assignment_stack[trail_position].reason {
                AssignmentReason::Forced(reason) => reason,
                AssignmentReason::Heuristic =>
                    panic!("Search should be completed by now. Trying to resolve with branching assignment"),
            }
        }

        // add the UIP
        learned_clause.push(-current_literal.unwrap());

        // The UIP has to be one of the watched literals. As the watches are initialized as 0 and 1
        // we swap the UIP into the first position.

        let learned_clause_len = learned_clause.len();
        learned_clause.swap(0, learned_clause_len - 1);
        debug_assert_eq!(
            trail.var_decision_level[learned_clause[0].id()],
            trail.decision_level
        );

        self.conflict_clause_minimization(&mut learned_clause, clause_database, trail, &seen);

        // learned clause is UIP
        debug_assert_eq!(
            learned_clause
                .iter()
                .filter(|lit| trail.var_decision_level[lit.id()] == trail.decision_level)
                .count(),
            1
        );

        //println!("shrunk: {} -> {}", len_before, learned_clause.len());

        // assertion level
        let assertion_level = learned_clause
            .clone()
            .iter()
            .map(|lit| trail.var_decision_level[lit.id()])
            .sorted()
            .rev()
            .nth(1)
            .unwrap_or(0);

        // The second watch has to be the asserting literal, otherwise the watched literals will
        // become invalid after backtracking. In unit clauses, there is no asserting literal and
        // this doesn't apply.
        if let Some(assert_lit_idx) = learned_clause
            .iter()
            .position(|lit| trail.var_decision_level[lit.id()] == assertion_level)
        {
            learned_clause.swap(1, assert_lit_idx);
        }

        assert!(assertion_level < trail.decision_level);

        // calculate lbd
        let lbd = learned_clause
            .iter()
            .map(|lit| trail.var_decision_level[lit.id()])
            .collect::<HashSet<_>>()
            .len();
        (
            Clause::from_literals_and_lbd(learned_clause, lbd),
            assertion_level,
        )
    }

    /// Conflict clause minimization based on Minisat v. 1.13
    fn conflict_clause_minimization(
        &self,
        clause: &mut Vec<Literal>,
        clause_database: &ClauseDatabase,
        trail: &Trail,
        seen: &HashSet<VarId, FastHasher>,
    ) {
        let mut minimized_clause = vec![clause[0]]; // keep the uip

        for literal in clause.iter().skip(1) {
            let reason_clause = match trail.get_reason(*literal) {
                AssignmentReason::Forced(reason_clause) => {
                    Some(&clause_database[*reason_clause].literals)
                }
                AssignmentReason::Heuristic => None,
            };

            if let Some(reason_clause) = reason_clause {
                for reason_literal in reason_clause.iter() {
                    if reason_literal.id() == literal.id() {
                        continue;
                    }
                    if !seen.contains(&reason_literal.id())
                        && trail.var_decision_level[reason_literal.id()] > 0
                    {
                        minimized_clause.push(*literal);

                        break;
                    }
                }
            } else {
                minimized_clause.push(*literal)
            }
        }
        *clause = minimized_clause;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::state::State;
    use crate::solver::trail::{Assignment, AssignmentReason};
    use crate::solver::unit_propagation::UnitPropagator;

    #[test]
    fn test_learn_clause() {
        let cnf = vec![
            Clause::from("-1 2"),      // 0
            Clause::from("-1 3 9"),    // 1
            Clause::from("-2 -3 4"),   // 2
            Clause::from("-4 5 10"),   // 3
            Clause::from("-4 6 11"),   // 4
            Clause::from("-5 -6"),     // 5
            Clause::from("1 7 -12"),   // 6
            Clause::from("1 8"),       // 7
            Clause::from("-7 -8 -13"), // 8
            Clause::from("10 -11"),    // 9
            Clause::from("-12 13"),    // 10
        ];
        let mut state = State::init(cnf.clone(), 13, false);
        let mut clause_learner = ClauseLearner::default();
        let mut brancher = Trail::new(13);
        let mut unit_propagator = UnitPropagator::default();

        let assigments = vec![-9, -10, 12, 1];
        // unit: 11
        for assignment in assigments {
            brancher.assign(
                &mut state,
                &mut unit_propagator,
                assignment.into(),
                AssignmentReason::Heuristic,
            );
            unit_propagator.propagate(&mut state, &mut brancher);
        }
        //state.verify_watches();
        assert_eq!(
            brancher.assignment_stack[0],
            Assignment::heuristic((-9).into(), 1)
        );
        assert_eq!(
            brancher.assignment_stack[5],
            Assignment::heuristic(1.into(), 4)
        );
        assert_eq!(
            brancher.assignment_stack[10],
            Assignment::forced(6.into(), 4, 4)
        );
        assert!(state.conflict_clause_id.is_some());
        // clause learning begins
        println!("{:?}", brancher.assignment_stack);
        let clause = clause_learner.analyse_conflict(
            &mut brancher,
            &mut state.clause_database,
            state.conflict_clause_id.clone().unwrap(),
        );
        println!("learned clause {:?}", clause);
        println!("{}", brancher.implication_graph(&state));
    }

    #[test]
    fn test_kit() {
        let cnf = vec![
            Clause::from("1 2"),      // 0
            Clause::from("2 3"),      // 1
            Clause::from("-1 -4 5"),  // 2
            Clause::from("-1 4 6"),   // 3
            Clause::from("-1 -5 6"),  // 4
            Clause::from("-1 4 -6"),  // 5
            Clause::from("-1 -5 -6"), // 6
        ];
        let mut state = State::init(cnf.clone(), 6, false);
        let mut clause_learner = ClauseLearner::default();
        let mut trail = Trail::new(state.num_vars);
        let mut unit_propagator = UnitPropagator::default();
        let assignments = vec![1, 2, 3, 4];
        for assignment in assignments {
            trail.assign(
                &mut state,
                &mut unit_propagator,
                assignment.into(),
                AssignmentReason::Heuristic,
            );
            unit_propagator.propagate(&mut state, &mut trail);
        }
        // state.verify_watches();
        println!("{}", trail.implication_graph(&state));
        assert!(state.conflict_clause_id.is_some());
        println!("{:?}", state.conflict_clause_id);
        println!("{:#?}", trail.assignment_stack);
        let learned_clause = clause_learner.analyse_conflict(
            &mut trail,
            &mut state.clause_database,
            state.conflict_clause_id.unwrap(),
        );
        println!("{:?}", learned_clause);
    }

    #[test]
    fn implication_graph() {
        let cnf = vec![
            Clause::from("-1 -2 -3"), // 0
            Clause::from("-2 -4 -5"), // 1
            Clause::from("3 5 6"),    // 2
            Clause::from("-6 -7"),    // 3
            Clause::from("-6 -8"),    // 4
            Clause::from("7 8"),      // 5
        ];
        let mut state = State::init(cnf.clone(), 8, false);
        let mut unit_propagator = UnitPropagator::default();
        let mut trail = Trail::new(state.num_vars);
        let mut clause_learner = ClauseLearner::default();
        let assignments = vec![1, 2, 4];
        for assignment in assignments {
            trail.assign(
                &mut state,
                &mut unit_propagator,
                assignment.into(),
                AssignmentReason::Heuristic,
            );
            unit_propagator.propagate(&mut state, &mut trail);
        }
        println!("{}", trail.implication_graph(&state));
        let learned_clause = clause_learner.analyse_conflict(
            &mut trail,
            &mut state.clause_database,
            state.conflict_clause_id.unwrap(),
        );
        println!("{:?}", learned_clause);
    }
}
