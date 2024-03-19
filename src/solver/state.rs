use crate::cnf::{Clause, ClauseId, Literal, VarId};
use crate::solver::clause_database::ClauseDatabase;
use crate::solver::literal_watching::{LiteralWatcher, WatchUpdate};
use crate::solver::statistics::StateStatistics;
use crate::solver::unit_propagation::UnitPropagator;
use itertools::Itertools;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct State {
    pub conflict_clause_id: Option<ClauseId>,
    pub vars: Vec<Option<bool>>,
    pub var_phases: Vec<bool>,
    pub clause_database: ClauseDatabase,
    pub literal_watcher: LiteralWatcher,
    pub num_vars: usize,
    pub stats: StateStatistics,
}

impl State {
    pub fn init(clauses: Vec<Clause>, n_vars: usize) -> Self {
        // remove tautologies
        let relevant_clauses = clauses
            .clone()
            .into_iter()
            .filter(|clause| {
                !clause
                    .literals
                    .iter()
                    .any(|lit| clause.literals.contains(&-*lit))
            })
            .collect_vec();

        State {
            conflict_clause_id: None,
            vars: vec![None; n_vars + 1],
            var_phases: vec![true; n_vars + 1],
            literal_watcher: LiteralWatcher::new(&relevant_clauses, n_vars),
            stats: StateStatistics::new(relevant_clauses.len(), n_vars),
            clause_database: ClauseDatabase::init(relevant_clauses.as_ref()),
            num_vars: n_vars,
        }
    }

    pub fn assign(&mut self, lit: Literal, unit_propagator: &mut UnitPropagator) {
        self.stats.num_assignments += 1;

        let (var_id, value) = lit.id_val();
        if self.vars[var_id].is_some() {
            panic!("Variable {} is already assigned!", var_id);
        }
        //self.verify_watches();
        self.vars[var_id] = Some(value);
        self.var_phases[var_id] = value;

        let affected_clauses = std::mem::take(self.literal_watcher.affected_clauses(lit));
        for clause_id in affected_clauses {
            // skip rest of clauses if conflict is detected
            if self.conflict_clause_id.is_some() {
                self.literal_watcher.add_watch(-lit, clause_id);
                continue;
            }

            let clause = &mut self.clause_database[clause_id];
            // check the blocking literal first
            if clause.check_blocking_literal(&self.vars) {
                self.literal_watcher.add_watch(-lit, clause_id);
                continue;
            }

            let watch_update = self
                .literal_watcher
                .update_clause(clause, clause_id, -lit, &self.vars);
            match watch_update {
                WatchUpdate::FoundNewWatch => {}
                WatchUpdate::Satisfied => {
                    self.literal_watcher.add_watch(-lit, clause_id);
                }
                WatchUpdate::Unit(unit) => {
                    unit_propagator.enqueue(unit, clause_id);
                }
                WatchUpdate::Conflict => {
                    self.conflict_clause_id = Some(clause_id);
                    self.stats.num_conflicts += 1;
                }
            }
        }
    }

    pub fn unassign(&mut self, lit: Literal) {
        self.vars[lit.id()] = None;
    }

    pub fn is_satisfied(&self) -> bool {
        self.clause_database
            .necessary_clauses_iter()
            .all(|clause_id| self.clause_database[clause_id].is_satisfied(&self.vars))
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
    pub fn verify_watches(&mut self) {
        for clause in self.clause_database.iter() {
            if self.clause_database[clause].is_satisfied(&self.vars) {
                continue;
            }
            if self.clause_database[clause].literals.len() == 1 {
                continue;
            }
            let watches = &self.clause_database[clause].literals[0..2];
            let zero = self.vars[watches[0].id()].is_none()
                || self.vars[watches[0].id()] == Some(watches[0].positive());
            let one = self.vars[watches[1].id()].is_none()
                || self.vars[watches[1].id()] == Some(watches[1].positive());
            assert!(zero || one || self.conflict_clause_id.is_some());
        }

        for clause_id in self.clause_database.iter() {
            if self.clause_database[clause_id].literals.len() == 1 {
                continue;
            }
            for lit in &self.clause_database[clause_id].literals[0..2] {
                assert!(
                    self.literal_watcher
                        .affected_clauses(-*lit)
                        .contains(&clause_id),
                    "Clause {} is not watched by {}",
                    clause_id,
                    lit
                );
            }
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
        //assert_eq!(state.clause_database.len(), 3);
    }

    #[test]
    fn test_state_assign() {
        let clauses = vec![Clause::from("1 2 3"), Clause::from("-1 -2 3")];
        let mut state = State::init(clauses);
        let mut unit_prop = UnitPropagator::default();
        state.assign(Literal::from(1), &mut unit_prop);
        assert_eq!(state.vars[1], Some(true));
        state.assign(Literal::from(2), &mut unit_prop);
        assert_eq!(state.vars[2], Some(true));
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

        assert_eq!(state.literal_watcher.var_watches[1].pos, vec![0]);
        assert_eq!(state.literal_watcher.var_watches[3].pos, vec![1]);
    }
}
