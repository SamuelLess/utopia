use crate::solver::branching::{Assignment, Brancher};
use crate::solver::state::State;

pub struct UnitPropagator {}

impl UnitPropagator {
    pub fn new() -> Self {
        UnitPropagator {}
    }

    pub fn propagate(&mut self, state: &mut State, brancher: &mut Brancher) {
        while let Some(lit) = state.unit_literals.pop_front() {
            let assignment = Assignment::forced(lit.id(), lit.positive());
            brancher.branch(state, assignment);
            if state.in_conflict {
                state.unit_literals.clear();
                return;
            }
        }
    }
}
