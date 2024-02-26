use crate::cnf::{Clause, ClauseId, Literal, VarId};
use crate::solver::literal_watching::{LiteralWatcher, WatchUpdate};
use crate::solver::statistics::StateStatistics;
use crate::solver::unit_propagation::UnitPropagator;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct State {
    pub conflict_clause_id: Option<ClauseId>,
    pub vars: Vec<Option<bool>>,
    pub clauses: Vec<Clause>,
    pub literal_watcher: LiteralWatcher,
    pub num_vars: usize,
    pub stats: StateStatistics,
}

impl State {
    pub fn init(clauses: Vec<Clause>) -> Self {
        let all_vars: HashSet<VarId> = clauses
            .clone()
            .into_iter()
            .flatten()
            .map(|lit| lit.id())
            .collect();

        State {
            conflict_clause_id: None,
            vars: vec![None; all_vars.len() + 1],
            clauses: clauses.clone(),
            literal_watcher: LiteralWatcher::new(&clauses, all_vars.len()),
            num_vars: all_vars.len(),
            stats: StateStatistics::new(clauses.len(), all_vars.len()),
        }
    }

    pub fn assign(&mut self, lit: Literal, unit_propagator: &mut UnitPropagator) {
        self.stats.num_assignments += 1;
        let (var_id, value) = lit.id_val();
        if self.vars[var_id].is_some() {
            panic!("Variable {} is already assigned!", var_id);
        }
        self.vars[var_id] = Some(value);

        let affected_clauses = self.literal_watcher.affected_clauses(lit);
        let mut changes: Vec<(ClauseId, [Literal; 2], [Literal; 2])> = vec![];
        for clause_id in affected_clauses {
            let clause = &mut self.clauses[*clause_id];
            // TODO: do this by using update clause to find if clause is satisfied
            // set one watch to the true literal
            if clause.is_satisfied(&self.vars) {
                continue;
            }
            let before = clause.watches();
            let watch_update = LiteralWatcher::update_clause(clause, &self.vars);
            changes.push((*clause_id, before, clause.watches()));
            match watch_update {
                WatchUpdate::Successful(_) => {}
                WatchUpdate::Unit(unit) => {
                    unit_propagator.enqueue(unit, *clause_id);
                }
                WatchUpdate::Conflict => {
                    self.conflict_clause_id = Some(*clause_id);
                    self.stats.num_conflicts += 1;
                    break;
                }
            }
        }
        self.literal_watcher.update_watches(&changes);
    }

    pub fn unassign(&mut self, lit: Literal) {
        self.vars[lit.id()] = None;
    }

    pub fn add_clause(&mut self, clause: Clause) -> ClauseId {
        let id = self.clauses.len();
        self.clauses.push(clause.clone());
        if clause.literals.len() != 1 {
            // watches are invalid unit will always be true
            self.literal_watcher.add_clause(&clause, id);
        }
        id
    }

    pub fn is_satisfied(&self) -> bool {
        self.clauses
            .iter()
            .all(|clause| clause.is_satisfied(&self.vars))
    }

    pub fn get_assignment(&self) -> HashMap<VarId, bool> {
        let mut result = HashMap::new();
        for (id, val) in self.vars.iter().enumerate().skip(1) {
            if let Some(val) = val {
                result.insert(id as VarId, *val);
            }
        }
        result
    }

    /// Verifies the watched literal invariant.
    /// Every unsatisfied clause has at least one watched literal
    /// that is non-false. If exactly one is non-false, it is
    /// queued for unit propagation. This leads to the clause
    /// being either satisfied or creating a conflict. Therefore
    /// the next steps either disregard the clause or both
    /// watched literals are non-false again.
    pub fn verify_watches(&self) {
        for clause in self.clauses.iter() {
            if clause.is_satisfied(&self.vars) {
                continue;
            }
            let watches = clause.watches();
            assert_eq!(watches[0].id(), clause.literals[clause.watches[0]].id());
            assert_eq!(watches[1].id(), clause.literals[clause.watches[1]].id());
            let zero = self.vars[watches[0].id()].is_none()
                || self.vars[watches[0].id()] == Some(watches[0].positive());
            let one = self.vars[watches[1].id()].is_none()
                || self.vars[watches[1].id()] == Some(watches[1].positive());
            assert!(zero || one || self.conflict_clause_id.is_some());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cnf::Clause;
    use crate::cnf::Literal;

    #[test]
    fn test_state_init() {
        let clauses = vec![
            Clause::from("1 2 3"),
            Clause::from("1 -2 3"),
            Clause::from("-1 -2 3"),
        ];
        let state = State::init(clauses);
        assert_eq!(state.num_vars, 3);
        assert_eq!(state.vars, vec![None, None, None, None]);
        assert_eq!(state.clauses.len(), 3);
    }

    #[test]
    fn test_state_assign() {
        let clauses = vec![Clause::from("1 2 3"), Clause::from("-1 -2 3")];
        let mut state = State::init(clauses);
        let mut unit_prop = UnitPropagator::default();
        state.assign(Literal::from(1), &mut unit_prop);
        assert_eq!(state.vars[1], Some(true));
        assert_eq!(state.clauses[0].watches, [0, 1]);
        assert_eq!(state.clauses[1].watches, [1, 2]);
        state.assign(Literal::from(2), &mut unit_prop);
        assert_eq!(state.vars[2], Some(true));
        assert_eq!(state.clauses[1].watches, [1, 2]);
        println!("{:?}", state);
        assert_eq!(unit_prop.unit_queue[0], (Literal::from(3), 1));
        state.assign(Literal::from(-3), &mut unit_prop);
        assert!(state.conflict_clause_id.is_some());
    }

    #[test]
    fn test_var_watches() {
        let clauses = vec![Clause::from("1 2 3"), Clause::from("-1 -2 3")];
        let mut state = State::init(clauses);
        let mut unit_prop = UnitPropagator::default();
        println!("{:?}", state);
        assert_eq!(state.literal_watcher.var_watches[1].pos, vec![0]);
        assert_eq!(state.literal_watcher.var_watches[1].neg, vec![1]);
        assert_eq!(state.literal_watcher.var_watches[3].pos, vec![]);

        state.assign(Literal::from(1), &mut unit_prop);
        state.assign(Literal::from(2), &mut unit_prop);
        println!("{:?}", state);
        assert!(state.clauses[0].watches.contains(&0));
        assert!(state.clauses[0].watches.contains(&1));
        assert!(state.clauses[1].watches.contains(&1));
        assert!(state.clauses[1].watches.contains(&2));

        assert_eq!(state.literal_watcher.var_watches[1].pos, vec![0]);
        assert_eq!(state.literal_watcher.var_watches[3].pos, vec![1]);
    }
}
