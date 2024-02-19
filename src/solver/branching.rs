use crate::cnf::VarId;
use crate::solver::state::State;

#[derive(Debug, Clone)]
pub struct Assignment {
    pub var: VarId,
    pub value: bool,
    pub reason: AssignmentReason,
}

impl Assignment {
    pub fn heuristic(var: VarId, value: bool) -> Self {
        Assignment {
            var,
            value,
            reason: AssignmentReason::Heuristic,
        }
    }

    pub fn forced(var: VarId, value: bool) -> Self {
        Assignment {
            var,
            value,
            reason: AssignmentReason::Forced,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignmentReason {
    Heuristic,
    Forced,
}

#[derive(Debug, Default, Clone)]
pub struct Brancher {
    pub assignment_stack: Vec<Assignment>,
}

impl Brancher {
    pub fn branch(&mut self, state: &mut State, assignment: Assignment) {
        self.push_assignment(assignment.clone());
        state.assign(assignment.into());
    }

    /// Backtrack to the last heuristic assignment
    /// and forces it to be the opposite value
    /// returns the forced assignment or none (implies unsat)
    pub fn backtrack(&mut self, state: &mut State) -> Option<Assignment> {
        let mut new_assignment: Option<Assignment> = None;
        while let Some(assignment) = self.assignment_stack.pop() {
            state.unassign(assignment.clone().into());
            if assignment.reason == AssignmentReason::Heuristic {
                new_assignment = Some(Assignment::forced(assignment.var, !assignment.value));
                break;
            }
        }
        state.in_conflict = false;
        state.assign(new_assignment.clone()?.into());
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
        println!("{:?}", state);
        let assignment1 = Assignment::heuristic(1, true);
        let assignment2 = Assignment::heuristic(2, true);
        brancher.branch(&mut state, assignment1);
        brancher.branch(&mut state, assignment2);
        brancher.backtrack(&mut state);
        assert_eq!(state.vars[1], Some(true));
        assert_eq!(state.vars[2], Some(false));
        brancher.backtrack(&mut state);
        assert_eq!(state.vars[1], Some(false));
        assert_eq!(state.vars[2], None);
    }
}
