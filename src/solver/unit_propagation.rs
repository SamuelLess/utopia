use crate::cnf::{ClauseId, Literal};
use crate::solver::state::State;
use crate::solver::trail::{AssignmentReason, Trail};
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Default)]
pub struct UnitPropagator {
    pub unit_queue: VecDeque<(Literal, ClauseId)>,
    pub units: HashSet<Literal>,
}

impl UnitPropagator {
    pub fn enqueue(&mut self, lit: Literal, reason: ClauseId) {
        // check if the literal is already in the queue
        if self.units.contains(&lit) {
            return;
        }
        self.unit_queue.push_back((lit, reason));
        self.units.insert(lit);
    }

    pub fn propagate(&mut self, state: &mut State, trail: &mut Trail) {
        while let Some((lit, clause_id)) = self.unit_queue.pop_front() {
            trail.assign(state, self, lit, AssignmentReason::Forced(clause_id));
            if state.conflict_clause_id.is_some() {
                self.unit_queue.clear();
                self.units.clear();
                return;
            }
        }
        self.units.clear();
    }
}
