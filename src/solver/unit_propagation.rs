use crate::solver::branching::{Assignment, Brancher};
use crate::solver::state::State;

pub struct UnitPropagator {}

impl UnitPropagator {
    pub fn new() -> Self {
        UnitPropagator {}
    }

    pub fn propagate(&mut self, state: &mut State, brancher: &mut Brancher) {
        while let Some(literal) = state.unit_literals.pop_front() {
            let assignment = Assignment::forced(literal.id(), literal.positive());
            brancher.branch(state, assignment);
        }
    }
}
