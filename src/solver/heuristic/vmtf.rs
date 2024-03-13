use crate::cnf::{Clause, Literal, VarId};
use crate::solver::heuristic::Heuristic;
use crate::solver::state::State;
use crate::solver::trail::Assignment;

#[derive(Default)]
pub struct HeuristicVMTF {
    pub order: Vec<VarId>,
}

impl Heuristic for HeuristicVMTF {
    fn init(state: &State) -> Self {
        // start out with all variables having a heuristic value of 1 and set to true
        HeuristicVMTF {
            order: (1..=state.vars.len()).collect(),
        }
    }

    fn unassign(&mut self, _assignment: &Assignment) {
        // no need to replay unassignments with VMTF
    }

    fn conflict(&mut self, clause: &Clause) {
        // remove the variables
        self.order
            .retain(|var_id| clause.literals.iter().all(|lit| lit.id() != *var_id));

        // add them to the front
        let var_ids = clause.literals.iter().map(|lit| lit.id());
        self.order = var_ids.chain(self.order.iter().cloned()).collect();
    }

    fn next(&mut self, vars: &[Option<bool>]) -> Literal {
        // find the first variable in the order that is not assigned
        let mut unassigned_pos = None;

        for var_id in &self.order {
            if vars[*var_id].is_none() {
                unassigned_pos = Some(*var_id);
                break;
            }
        }

        Literal::from_value(unassigned_pos.expect("No unassigned variable found"), true)
    }
}
