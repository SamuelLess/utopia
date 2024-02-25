use crate::cnf::{Clause, ClauseId};
use crate::solver::branching::{AssignmentReason, Brancher};

#[derive(Debug, Default, Clone)]
pub struct ClauseLearner {}

impl ClauseLearner {
    /// Assumes that the current state is in conflict
    pub fn learn_clause(
        &mut self,
        brancher: &mut Brancher,
        clauses: &[Clause],
        conflict_clause_id: ClauseId,
    ) -> Clause {
        let decision_level_literals = brancher
            .assignment_stack
            .iter()
            .filter(|a| a.decision_level == brancher.decision_level)
            .map(|a| a.literal)
            .collect::<Vec<_>>();

        let mut learned_clause = clauses[conflict_clause_id].clone();
        for assignment in brancher.assignment_stack.iter().rev() {
            let left_decision_level_literals = learned_clause.literals.iter().filter(|lit| {
                decision_level_literals.contains(lit) || decision_level_literals.contains(&-**lit)
            });
            if left_decision_level_literals.count() <= 1 {
                break;
            }
            let old_clause = learned_clause.clone();
            if let AssignmentReason::Forced(reason) = assignment.reason {
                let reason_clause = &clauses[reason];
                learned_clause = learned_clause.resolution(reason_clause.clone());
            }
            if learned_clause.literals.len() <= 1 {
                return old_clause;
            }
        }
        learned_clause
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::branching::{Assignment, AssignmentReason};
    use crate::solver::state::State;
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
        let mut brancher = Brancher::default();
        let mut unit_propagator = UnitPropagator::default();

        let assigments = vec![-9, -10, 12, 1];
        // unit: 11
        for assignment in assigments {
            brancher.branch(
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
        let clause = clause_learner.learn_clause(
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
        let mut brancher = Brancher::default();
        let mut unit_propagator = UnitPropagator::default();
        let assignments = vec![1, 2, 3, 4];
        for assignment in assignments {
            brancher.branch(
                &mut state,
                &mut unit_propagator,
                assignment.into(),
                AssignmentReason::Heuristic,
            );
            unit_propagator.propagate(&mut state, &mut brancher);
        }
        state.verify_watches();
        println!("{}", brancher.implication_graph(&state));
        assert!(state.conflict_clause_id.is_some());
        println!("{:?}", state.conflict_clause_id);
        println!("{:#?}", brancher.assignment_stack);
        let clause = clause_learner.learn_clause(
            &mut brancher,
            &state.clauses,
            state.conflict_clause_id.unwrap(),
        );
        println!("learned clause {:?}", clause);
    }
}
