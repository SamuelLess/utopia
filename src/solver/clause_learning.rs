use crate::cnf::{Clause, ClauseId};
use crate::solver::trail::{AssignmentReason, Trail};
use itertools::Itertools;

#[derive(Debug, Default, Clone)]
pub struct ClauseLearner {}

impl ClauseLearner {
    /// Assumes that the current state is in conflict
    pub fn analyse_conflict(
        &mut self,
        trail: &mut Trail,
        clauses: &[Clause],
        conflict_clause_id: ClauseId,
    ) -> (Clause, usize) {
        let decision_level_literals = trail
            .assignment_stack
            .iter()
            .filter(|a| a.decision_level == trail.decision_level)
            .map(|a| a.literal)
            .collect::<Vec<_>>();

        let mut learned_clause = clauses[conflict_clause_id].clone();
        for assignment in trail.assignment_stack.iter().rev() {
            let left_decision_level_literals = learned_clause.literals.iter().filter(|lit| {
                decision_level_literals.contains(lit) || decision_level_literals.contains(&-**lit)
            });
            if left_decision_level_literals.count() <= 1 {
                break;
            }
            if let AssignmentReason::Forced(reason) = assignment.reason {
                let reason_clause = &clauses[reason];
                learned_clause = learned_clause.resolution(reason_clause.clone());
            }
        }
        // assertion level
        let assertion_level = learned_clause
            .clone()
            .map(|lit| {
                trail
                    .assignment_stack
                    .iter()
                    .find(|a| a.literal.id() == lit.id())
                    .unwrap()
                    .decision_level
            })
            .sorted()
            .rev()
            .nth(1)
            .unwrap_or(0);
        assert!(assertion_level < trail.decision_level);
        (learned_clause, assertion_level)
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
        let mut state = State::init(cnf.clone());
        let mut clause_learner = ClauseLearner::default();
        let mut brancher = Trail::default();
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
        state.verify_watches();
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
            &state.clauses,
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
        let mut state = State::init(cnf.clone());
        let mut clause_learner = ClauseLearner::default();
        let mut trail = Trail::default();
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
        state.verify_watches();
        println!("{}", trail.implication_graph(&state));
        assert!(state.conflict_clause_id.is_some());
        println!("{:?}", state.conflict_clause_id);
        println!("{:#?}", trail.assignment_stack);
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
        let mut state = State::init(cnf.clone());
        let mut clause_learner = ClauseLearner::default();
        let mut unit_propagator = UnitPropagator::default();
        let mut trail = Trail::default();
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
            &state.clauses,
            state.conflict_clause_id.unwrap(),
        );
        println!("{:?}", learned_clause);
    }
}
