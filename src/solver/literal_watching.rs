use crate::cnf::{Clause, ClauseId, Literal};

#[derive(Debug, Default, Clone)]
pub struct VarWatch {
    pub pos: Vec<ClauseId>,
    pub neg: Vec<ClauseId>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WatchUpdate {
    Successful([Literal; 2]),
    Unit(Literal),
    Conflict,
}

#[derive(Debug, Clone)]
pub struct LiteralWatcher {
    pub var_watches: Vec<VarWatch>,
}

impl LiteralWatcher {
    pub fn new(clauses: &[Clause], num_vars: usize) -> Self {
        LiteralWatcher {
            var_watches: Self::create_watches(clauses, num_vars),
        }
    }

    pub fn add_clause(&mut self, clause: &Clause, clause_id: ClauseId) {
        for lit in clause.watches() {
            self.add_watch(lit, clause_id);
        }
    }

    pub fn affected_clauses(&self, lit: Literal) -> &[ClauseId] {
        if lit.positive() {
            &self.var_watches[lit.id()].neg
        } else {
            &self.var_watches[lit.id()].pos
        }
    }

    /// TODO: Find a more efficient way to update watched literals.
    /// This 'batched updating' is done to not copy the affected clauses to iterate.
    pub fn update_watches(&mut self, updates: &[(ClauseId, [Literal; 2], [Literal; 2])]) {
        for (clause_id, before, after) in updates {
            // remove clause id form before update
            self.remove_watch(before[0], *clause_id);
            self.remove_watch(before[1], *clause_id);
            // add clause_id to from after update
            self.add_watch(after[0], *clause_id);
            self.add_watch(after[1], *clause_id);
        }
    }

    fn remove_watch(&mut self, lit: Literal, clause_id: ClauseId) {
        if lit.positive() {
            self.var_watches[lit.id()].pos.retain(|&x| x != clause_id);
        } else {
            self.var_watches[lit.id()].neg.retain(|&x| x != clause_id);
        }
    }

    fn add_watch(&mut self, lit: Literal, clause_id: ClauseId) {
        if lit.positive() {
            self.var_watches[lit.id()].pos.push(clause_id);
        } else {
            self.var_watches[lit.id()].neg.push(clause_id);
        }
    }

    /// Sets watched literal to two non-false literals if possible
    /// Otherwise the clause is unit and the unit literal is returned as well
    /// Returns the new watches and whether the clause is unit
    pub fn update_clause(clause: &mut Clause, vars: &[Option<bool>]) -> WatchUpdate {
        let possible_watches = clause.possible_watches_idx(vars);
        if possible_watches.len() >= 2 {
            clause.watches = [possible_watches[0], possible_watches[1]];
            return WatchUpdate::Successful(clause.watches());
        }
        if possible_watches.len() == 1 {
            let unit_idx = possible_watches[0];
            let update_idx = if clause.watches[0] == unit_idx { 0 } else { 1 };
            clause.watches[update_idx] = unit_idx;
            return WatchUpdate::Unit(clause.literals[unit_idx]);
        }
        WatchUpdate::Conflict
    }
    fn create_watches(clauses: &[Clause], num_vars: usize) -> Vec<VarWatch> {
        let mut watches = vec![VarWatch::default(); num_vars + 1];
        for (clause_id, clause) in clauses.iter().enumerate() {
            for lit in clause.watches() {
                let var_id = lit.id();
                if lit.positive() {
                    watches[var_id].pos.push(clause_id);
                } else {
                    watches[var_id].neg.push(clause_id);
                }
            }
        }
        watches
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::state::State;
    use crate::solver::unit_propagation::UnitPropagator;

    #[test]
    fn test_update_watches_clauses() {
        let mut clause = Clause::from("1 2 3");
        let mut assignment = Vec::from([None; 4]);
        assignment.insert(1, Some(false));
        assert!(!clause.is_satisfied(&assignment));
        LiteralWatcher::update_clause(&mut clause, &assignment);
        assert_eq!(clause.watches, [1, 2]);
        assignment.insert(2, Some(false));
        let update = LiteralWatcher::update_clause(&mut clause, &assignment);
        // unit 3 is on index 2 in clause
        assert_eq!(update, WatchUpdate::Unit(Literal::from(3)));
        assert_eq!(clause.watches, [1, 2]);
    }

    #[test]
    fn test_update_watches() {
        let clauses = vec![Clause::from("1 2 3"), Clause::from("-1 -2 3 4")];
        let mut state = State::init(clauses);
        let mut unit_prop = UnitPropagator::default();
        state.assign(Literal::from(1), &mut unit_prop);
        assert_eq!(state.clauses[0].watches, [0, 1]);
        assert_eq!(state.clauses[1].watches, [1, 2]);
        println!("{:?}", state);
        assert_eq!(state.literal_watcher.var_watches[1].neg, vec![]);
        assert_eq!(state.literal_watcher.var_watches[3].pos, vec![1]);
        state.assign(Literal::from(2), &mut unit_prop);
        assert_eq!(state.literal_watcher.var_watches[3].pos, vec![1]);
        println!("{:?}", state);
        state.unassign(Literal::from(2));
        state.assign(Literal::from(-3), &mut unit_prop);
        assert_eq!(state.literal_watcher.var_watches[3].pos, vec![]);
        assert_eq!(state.literal_watcher.var_watches[3].neg, vec![]);
        println!("{:?}", state);
    }

    #[test]
    fn test_update_watches_2() {
        let clauses = vec![Clause::from("-1 -2 3")];
        let mut state = State::init(clauses);
        let mut unit_prop = UnitPropagator::default();
        state.assign(Literal::from(-1), &mut unit_prop);
        state.assign(Literal::from(2), &mut unit_prop);
        state.unassign(Literal::from(2));
        state.unassign(Literal::from(-1));
        state.assign(Literal::from(1), &mut unit_prop);
        state.assign(Literal::from(2), &mut unit_prop);
        state.verify_watches();
        println!("{:?}", state);
    }
}
