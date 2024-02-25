use crate::cnf::{Clause, ClauseId, Literal, VarId};
use crate::solver::state::State;
use crate::solver::unit_propagation::UnitPropagator;

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub literal: Literal,
    pub reason: AssignmentReason,
    pub decision_level: usize,
}

impl Assignment {
    pub fn heuristic(literal: Literal, decision_level: usize) -> Self {
        Assignment {
            literal,
            reason: AssignmentReason::Heuristic,
            decision_level,
        }
    }

    pub fn forced(literal: Literal, decision_level: usize, reason: ClauseId) -> Self {
        Assignment {
            literal,
            reason: AssignmentReason::Forced(reason),
            decision_level,
        }
    }
}

/// AssignmentReason, Heuristic contains decision level
#[derive(Debug, Clone, PartialEq)]
pub enum AssignmentReason {
    Heuristic,
    Forced(ClauseId),
}

#[derive(Debug, Default, Clone)]
pub struct Brancher {
    pub assignment_stack: Vec<Assignment>,
    pub decision_level: usize,
}

impl Brancher {
    pub fn branch(
        &mut self,
        state: &mut State,
        unit_propagator: &mut UnitPropagator,
        literal: Literal,
        reason: AssignmentReason,
    ) {
        // increase before because every
        // forced assigment is at the same level
        if reason == AssignmentReason::Heuristic {
            self.decision_level += 1;
        }
        let assignment = Assignment {
            literal,
            reason,
            decision_level: self.decision_level,
        };
        self.push_assignment(assignment.clone());
        state.assign(assignment.into(), unit_propagator);
    }

    /// Backtrack to the last heuristic assignment
    /// and forces it to be the opposite value
    /// returns the forced assignment or none (implies unsat)
    pub fn backtrack(
        &mut self,
        state: &mut State,
        unit_propagator: &mut UnitPropagator,
        conflict_clause: ClauseId,
    ) -> Option<Assignment> {
        self.decision_level -= 1;
        let mut new_assignment: Option<Assignment> = None;
        while let Some(assignment) = self.assignment_stack.pop() {
            state.unassign(assignment.clone().into());
            if assignment.reason == AssignmentReason::Heuristic {
                new_assignment = Some(Assignment::forced(
                    -assignment.literal,
                    self.decision_level,
                    conflict_clause,
                ));
                break;
            }
        }
        state.conflict_clause_id = None;
        state.assign(new_assignment.clone()?.into(), unit_propagator);
        self.push_assignment(new_assignment.clone()?);
        new_assignment
    }

    /// Returns the assignments that from top to most recent heuristic
    pub fn assignments_to_undo(&self) -> &[Assignment] {
        // find the last heuristic assignment
        let last = self
            .assignment_stack
            .iter()
            .rev()
            .position(|assignment| assignment.reason == AssignmentReason::Heuristic)
            .unwrap_or(0);
        let len = self.assignment_stack.len();
        &self.assignment_stack[(len - last)..]
    }

    pub fn push_assignment(&mut self, assignment: Assignment) {
        self.assignment_stack.push(assignment);
    }

    pub fn implication_graph(&self, state: &State) -> String {
        let mut out = String::from("digraph G {\n");
        for assignment in self.assignment_stack.iter() {
            if let AssignmentReason::Forced(reason) = &assignment.reason {
                for lit in &state.clauses[*reason].literals {
                    if lit == &assignment.literal {
                        continue;
                    }
                    out.push_str(&format!(
                        "{} -> {} [label=\"{}\"];\n",
                        -*lit, assignment.literal, reason
                    ));
                }
            }
        }
        if let Some(conflict_clause_id) = &state.conflict_clause_id {
            for lit in &state.clauses[*conflict_clause_id].literals {
                out.push_str(&format!("{} -> C [color=red, label=\"\"];\n", -*lit));
            }
        }
        out.push_str("}\n");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cnf::Clause;

    #[test]
    fn test_backtrack() {
        let mut brancher = Brancher::default();
        let clauses = vec![Clause::from("1 2 3"), Clause::from("-1 -2 3")];
        let mut state = State::init(clauses);
        let mut unit_prop = UnitPropagator::default();
        println!("{:?}", state);
        let assignment1 = Assignment::heuristic(1.into(), 1);
        let assignment2 = Assignment::heuristic(2.into(), 2);
        brancher.branch(
            &mut state,
            &mut unit_prop,
            assignment1.clone().into(),
            AssignmentReason::Heuristic,
        );
        brancher.branch(
            &mut state,
            &mut unit_prop,
            assignment2.clone().into(),
            AssignmentReason::Heuristic,
        );
        brancher.backtrack(&mut state, &mut unit_prop, 0);
        assert_eq!(state.vars[1], Some(true));
        assert_eq!(state.vars[2], Some(false));
        brancher.backtrack(&mut state, &mut unit_prop, 0);
        assert_eq!(state.vars[1], Some(false));
        assert_eq!(state.vars[2], None);
    }
}
