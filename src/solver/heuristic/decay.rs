use crate::cnf::{Literal, VarId};
use crate::solver::branching::Assignment;
use crate::solver::heuristic::Heuristic;
use crate::solver::state::State;
use itertools::Itertools;

#[derive(Default)]
pub struct HeuristicDecay {
    pub order: Vec<(VarId, bool, f64)>,

    pub positions: Vec<usize>,

    pub branch_count: usize,
}

impl HeuristicDecay {
    fn recalc_positions(&mut self) {
        for (i, (var_id, _, _)) in self.order.iter().enumerate() {
            self.positions[*var_id] = i;
        }
    }

    pub fn initialize(&mut self, state: &State) {
        // start out with all variables having a heuristic value of 1 and set to true
        self.order = (0..state.vars.len())
            .map(|id| (id, true, 1.0))
            .collect_vec();

        self.positions = (0..state.vars.len()).collect_vec();

        self.recalc_positions();
    }

    pub fn choose_literal(&mut self, vars: &[Option<bool>]) -> Literal {
        self.branch_count += 1;

        // decay the heuristic values every 100 branches
        if self.branch_count > 100 {
            self.branch_count = 0;
            self.order = self
                .order
                .iter()
                .map(|(id, sign, heuristic_value)| {
                    let new_heuristic_value = heuristic_value * 0.95;
                    (*id, *sign, new_heuristic_value)
                })
                .collect_vec();

            // sort the list by the heuristic value
            self.order
                .sort_by(|(_, _, heuristic_value1), (_, _, heuristic_value2)| {
                    heuristic_value2.partial_cmp(heuristic_value1).unwrap()
                });

            self.recalc_positions();
        }

        // return the first element that is not assigned
        for (var_id, sign, _) in &self.order {
            if vars[*var_id].is_none() {
                return Literal::from_value(*var_id, *sign);
            }
        }
        panic!("No unassigned literal found");
    }

    pub fn unassigned_var(&mut self, var: VarId) {
        // increase the key of the var by one
        let (var_id, _, heuristic_value) = &mut self.order[self.positions[var]];
        debug_assert_eq!(*var_id, var);
        *heuristic_value += 1.0;
    }
}

impl Heuristic for HeuristicDecay {
    fn init(state: &State) -> Self {
        let mut manager = HeuristicDecay::default();
        manager.initialize(state);
        manager
    }

    fn replay_unassignments(&mut self, assignments: &[Assignment]) {
        for assignment in assignments {
            self.unassigned_var(assignment.var);
        }
    }

    fn next(&mut self, vars: &[Option<bool>]) -> Assignment {
        let literal = self.choose_literal(vars);
        Assignment::heuristic(literal.id(), literal.positive())
    }
}
