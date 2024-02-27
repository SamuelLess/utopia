use crate::cnf::{ClauseId, Literal};
use crate::solver::clause_learning::ClauseLearner;
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
pub struct Trail {
    pub assignment_stack: Vec<Assignment>,
    pub decision_level: usize,
}

impl Trail {
    pub fn assign(
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
        learned_clause_id: ClauseId,
        assertion_level: usize,
    ) {
        let learned_clause = state.clauses[learned_clause_id].clone();

        let top = learned_clause
            .clone()
            .find(|lit| {
                let assignment = self
                    .assignment_stack
                    .iter()
                    .find(|a| a.literal.id() == lit.id())
                    .unwrap();
                assignment.decision_level == self.decision_level
            })
            .expect("Clause was not UIP");
        // top most element is in the conflict clause and has highest decision level
        unit_propagator.enqueue(top, learned_clause_id);
        while let Some(assignment) = self.assignment_stack.last().cloned() {
            if assignment.decision_level == assertion_level {
                break;
            }
            self.assignment_stack.pop();
            state.unassign(assignment.literal);
        }
        self.decision_level = assertion_level;
        state.conflict_clause_id = None;
    }

    /// Returns the assignments that from top to most recent heuristic
    pub fn assignments_to_undo(&self, assertion_level: usize) -> &[Assignment] {
        // find the last heuristic assignment
        let last = self
            .assignment_stack
            .iter()
            .rev()
            .position(|assignment| assignment.decision_level == assertion_level)
            .unwrap_or(0);
        // [1@1,7@1,2@2,3@2,4@3,5@3] -> 3 idx = 2 ->  len=6-2-1=4
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
                    let dl_lit = ClauseLearner::get_decision_level(*lit, &self);
                    let dl_re =
                        ClauseLearner::get_decision_level(assignment.literal.clone(), &self);
                    out.push_str(&format!(
                        "\"{}@{}\" -> \"{}@{}\" [label=\"{}\"];\n",
                        -*lit, dl_lit, assignment.literal, dl_re, reason,
                    ));
                }
            }
        }
        if let Some(conflict_clause_id) = &state.conflict_clause_id {
            for lit in &state.clauses[*conflict_clause_id].literals {
                let dl_lit = ClauseLearner::get_decision_level(*lit, &self);
                out.push_str(&format!(
                    "\"{}@{}\" -> C [color=red, label=\"\"];\n",
                    -*lit, dl_lit
                ));
            }
        }
        out.push_str("}\n");
        out
    }
}
