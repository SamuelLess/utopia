use crate::cnf::{Literal, VarId};
use crate::solver::heuristic::Heuristic;
use crate::solver::state::State;
use crate::solver::trail::Assignment;

pub struct HeuristicTrue {
    pub order: Vec<(VarId, bool)>,
}

impl Heuristic for HeuristicTrue {
    fn init(state: &State) -> Self {
        let order = (1..=state.num_vars).map(|id| (id, true)).collect();
        // reverse
        HeuristicTrue { order }
    }

    fn unassign(&mut self, assignment: &Assignment) {
        self.order.push(assignment.literal.id_val());
    }

    fn next(&mut self, vars: &[Option<bool>]) -> Literal {
        for (id, val) in self.order.iter() {
            if vars[*id].is_none() {
                return Literal::from_value(*id, *val);
            }
        }
        panic!("No unassigned literal found");
    }
}
