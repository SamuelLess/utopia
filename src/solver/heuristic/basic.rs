use crate::cnf::VarId;
use crate::solver::branching::Assignment;
use crate::solver::heuristic::Heuristic;
use crate::solver::state::State;

pub struct HeuristicTrue {
    pub order: Vec<(VarId, bool)>,
}

impl Heuristic for HeuristicTrue {
    fn init(state: &State) -> Self {
        let order = (1..=state.num_vars).map(|id| (id, true)).collect();
        // reverse
        HeuristicTrue { order }
    }

    fn replay_unassignments(&mut self, assignments: &[Assignment]) {
        for assignment in assignments {
            self.order.push((assignment.var, assignment.value));
        }
    }

    fn next(&mut self, vars: &[Option<bool>]) -> Assignment {
        for (id, val) in self.order.iter() {
            if vars[*id].is_none() {
                return Assignment::heuristic(*id, *val);
            }
        }
        panic!("No unassigned literal found");
    }
}
