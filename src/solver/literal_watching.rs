use crate::cnf::{Clause, ClauseId, Literal};

#[derive(Debug, Default, Clone)]
pub struct VarWatch {
    pub pos: Vec<ClauseId>,
    pub neg: Vec<ClauseId>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WatchUpdate {
    FoundNewWatch,
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

    pub fn affected_clauses(&mut self, lit: Literal) -> &mut Vec<ClauseId> {
        if lit.positive() {
            &mut self.var_watches[lit.id()].neg
        } else {
            &mut self.var_watches[lit.id()].pos
        }
    }

    pub fn add_watch(&mut self, lit: Literal, clause_id: ClauseId) {
        if lit.positive() {
            self.var_watches[lit.id()].pos.push(clause_id);
        } else {
            self.var_watches[lit.id()].neg.push(clause_id);
        }
    }

    /// Sets watched literal to two non-false literals if possible
    pub fn update_clause(
        &mut self,
        clause: &mut Clause,
        clause_id: ClauseId,
        old_literal: Literal,
        vars: &[Option<bool>],
    ) -> WatchUpdate {
        let mut watched_literals = clause.watches();
        // ensure that the first watch is the newly false one
        if watched_literals[0].id() != old_literal.id() {
            clause.watches.swap(0, 1);
            watched_literals.swap(0, 1);
        }
        assert_eq!(watched_literals[0], old_literal);

        assert!(old_literal.is_false(vars));

        if watched_literals[1].is_false(vars) {
            self.add_watch(old_literal, clause_id);
            return WatchUpdate::Conflict;
        }

        for i in 0..clause.literals.len() {
            if i == clause.watches[1] {
                continue;
            }
            if clause.literals[i].non_false(vars) {
                // Found new valid watch
                clause.watches[0] = i;
                self.add_watch(clause.literals[i], clause_id);
                return WatchUpdate::FoundNewWatch;
            }
        }
        // As the entire watchlist of this literal is getting cleared, we need to re-add it
        // if it's a unit and we don't actually change the literal
        self.add_watch(old_literal, clause_id);
        WatchUpdate::Unit(clause.literals[clause.watches[1]])
    }

    fn create_watches(clauses: &[Clause], num_vars: usize) -> Vec<VarWatch> {
        let mut watches = vec![VarWatch::default(); num_vars + 1];
        for (clause_id, clause) in clauses.iter().enumerate() {
            if clause.literals.len() == 1 {
                continue; // Don't watch unit clauses, they never change
            }

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
