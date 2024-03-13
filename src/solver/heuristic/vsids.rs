use ordered_float::NotNan;
use priority_queue::PriorityQueue;

use crate::cnf::{Clause, Literal, VarId};
use crate::solver::heuristic::Heuristic;
use crate::solver::state::State;
use crate::solver::trail::Assignment;

#[derive(Default)]
pub struct HeuristicVSIDS {
    pub order: PriorityQueue<VarId, NotNan<f64>>,
    priorities: Vec<NotNan<f64>>,
    conflict_index: f64,
}

const BUMP_BASIS: f64 = 1.1;

impl HeuristicVSIDS {
    fn rescale(&mut self, factor: f64) {
        // divide everything by factor
        for priority in &mut self.priorities {
            *priority = NotNan::new(priority.into_inner() / factor).unwrap();
        }
        // change conflict index, such that BUMP_FACTOR^conflict_index gets divided by factor
        // g^i_new = g^i_old / factor

        self.conflict_index -= factor.ln() / BUMP_BASIS.ln();

        let mut new_order = PriorityQueue::new();

        // rescale the priorities in the queue
        for (var_id, _) in self.order.clone() {
            new_order.push(var_id, self.priorities[var_id]);
        }
        self.order = new_order;
    }
}

impl Heuristic for HeuristicVSIDS {
    fn init(state: &State) -> Self {
        // start out with all variables having a heuristic value of 1 and set to true
        HeuristicVSIDS {
            priorities: vec![NotNan::new(1.0).unwrap(); state.vars.len() + 1],
            order: (1..state.vars.len())
                .map(|id| (id, NotNan::new(1.0).unwrap()))
                .collect(),
            conflict_index: 0.0,
        }
    }

    fn unassign(&mut self, assignment: &Assignment) {
        let (var_id, _) = assignment.literal.id_val();
        self.order.push(var_id, self.priorities[var_id]); // replaces any existing priority
    }

    fn conflict(&mut self, clause: &Clause) {
        self.conflict_index += 1.0;

        // bump the priority of the variables in the clause
        for lit in &clause.literals {
            let (var_id, _) = lit.id_val();

            let mut increase = BUMP_BASIS.powi(self.conflict_index as i32);
            let new_priority = self.priorities[var_id].into_inner() + increase;
            if new_priority.is_infinite() {
                self.rescale(self.priorities[var_id].into_inner());
                increase = BUMP_BASIS.powi(self.conflict_index as i32);
            }

            self.priorities[var_id] =
                NotNan::new(self.priorities[var_id].into_inner() + increase).unwrap();
            self.order.change_priority(&var_id, self.priorities[var_id]);
        }
    }

    fn next(&mut self, vars: &[Option<bool>]) -> Literal {
        loop {
            let (var_id, _) = self.order.pop().expect("No unassigned variable found");
            if vars[var_id].is_none() {
                return Literal::from_value(var_id, true);
            }
        }
    }
}
