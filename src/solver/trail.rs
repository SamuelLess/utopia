use crate::cnf::{ClauseId, Literal};
use crate::solver::heuristic::Heuristic;
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

#[derive(Debug, Clone)]
pub struct Trail {
    pub assignment_stack: Vec<Assignment>,
    pub var_decision_level: Vec<usize>,
    pub var_assignment_pos: Vec<usize>,
    pub decision_level: usize,
}

impl Trail {
    pub fn new(num_vars: usize) -> Trail {
        Trail {
            assignment_stack: vec![],
            var_decision_level: vec![0; num_vars + 1],
            var_assignment_pos: vec![0; num_vars + 1],
            decision_level: 0,
        }
    }
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
        self.var_decision_level[literal.id()] = self.decision_level;
        self.var_assignment_pos[literal.id()] = self.assignment_stack.len() - 1;

        state.assign(assignment.into(), unit_propagator);
    }

    /// Backtracks completely, including the unit clause forced assignments
    /// This is necessary for inprocessing
    pub fn backtrack_completely(&mut self, state: &mut State, heuristic: &mut dyn Heuristic) {
        while let Some(assignment) = self.assignment_stack.pop() {
            state.unassign(assignment.literal);
            heuristic.unassign(&assignment);
        }
        self.decision_level = 0;
        state.conflict_clause_id = None;
    }

    /// Backtrack to the last heuristic assignment
    /// and forces it to be the opposite value
    /// returns the forced assignment or none (implies unsat)
    pub fn backtrack(
        &mut self,
        state: &mut State,
        heuristic: &mut dyn Heuristic,
        assertion_level: usize,
    ) {
        while let Some(assignment) = self.assignment_stack.last().cloned() {
            if assignment.decision_level == assertion_level {
                break;
            }
            heuristic.unassign(&assignment);
            self.assignment_stack.pop();
            state.unassign(assignment.literal);
        }

        self.decision_level = assertion_level;
        state.conflict_clause_id = None;
    }

    pub fn restart(&mut self, state: &mut State, heuristic: &mut dyn Heuristic) {
        self.backtrack(state, heuristic, 0);
    }

    pub fn push_assignment(&mut self, assignment: Assignment) {
        self.assignment_stack.push(assignment);
    }

    pub fn get_reason(&self, literal: Literal) -> &AssignmentReason {
        let pos = self.var_assignment_pos[literal.id()];
        &self.assignment_stack[pos].reason
    }

    pub fn implication_graph(&self, state: &State) -> String {
        let mut out = String::from("digraph G {\n");
        for assignment in self.assignment_stack.iter() {
            if let AssignmentReason::Forced(reason) = &assignment.reason {
                for lit in &state.clause_database[*reason].literals {
                    if lit == &assignment.literal {
                        continue;
                    }
                    let dl_lit = self.var_decision_level[lit.id()];
                    let dl_re = self.var_decision_level[assignment.literal.id()];
                    out.push_str(&format!(
                        "\"{}@{}\" -> \"{}@{}\" [label=\"{}\"];\n",
                        -*lit, dl_lit, assignment.literal, dl_re, reason,
                    ));
                }
            }
        }
        if let Some(conflict_clause_id) = &state.conflict_clause_id {
            for lit in &state.clause_database[*conflict_clause_id].literals {
                let dl_lit = self.var_decision_level[lit.id()];
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
