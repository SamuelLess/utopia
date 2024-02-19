use crate::cnf::{Clause, ClauseId, Literal, VarId};
use crate::solver::statistics::StateStatistics;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Default, Clone)]
pub struct VarWatches {
    pub pos: Vec<ClauseId>,
    pub neg: Vec<ClauseId>,
}

#[derive(Debug, Clone)]
pub struct State {
    pub in_conflict: bool,
    pub vars: Vec<Option<bool>>,
    pub clauses: Vec<Clause>,
    pub var_watches: Vec<VarWatches>,
    pub num_vars: usize,
    pub unit_literals: VecDeque<Literal>,
    pub stats: StateStatistics,
}

impl State {
    pub fn init(clauses: Vec<Clause>) -> Self {
        //let mut num_vars: Vec<VarId> = clauses.iter().flatten().map(|lit| lit.id()).collect();
        let all_vars: HashSet<VarId> = clauses
            .clone()
            .into_iter()
            .flatten()
            .map(|lit| lit.id())
            .collect();

        let mut var_watches = vec![VarWatches::default(); all_vars.len() + 1];

        for (clause_id, clause) in clauses.clone().into_iter().enumerate() {
            for literal in clause.watches() {
                let var_id = literal.id();
                if literal.positive() {
                    var_watches[var_id].pos.push(clause_id);
                } else {
                    var_watches[var_id].neg.push(clause_id);
                }
            }
        }

        State {
            in_conflict: false,
            vars: vec![None; all_vars.len() + 1],
            clauses: clauses.clone(),
            var_watches,
            num_vars: all_vars.len(),
            unit_literals: VecDeque::new(),
            stats: StateStatistics::new(clauses.len(), all_vars.len()),
        }
    }

    pub fn assign(&mut self, lit: Literal) {
        self.stats.num_assignments += 1;
        let var_id = lit.id();
        let value = lit.positive();
        self.vars[var_id] = Some(value);

        // find new watches for false literals
        let affected_clauses = if value {
            &self.var_watches[var_id].neg
        } else {
            &self.var_watches[var_id].pos
        };
        let mut changes: Vec<(ClauseId, Literal)> = vec![];
        for clause_id in affected_clauses {
            let clause = &mut self.clauses[*clause_id];
            if clause.is_satisfied(&self.vars) {
                continue;
            }
            let watches_literal = clause.watches();
            let update_first = watches_literal[0].id() == var_id;
            let new_watch = clause.literals.iter().enumerate().find(|(_, lit)| {
                let id = lit.id();
                id != watches_literal[0].id()
                    && id != watches_literal[1].id()
                    && self.vars[id].is_none()
            });
            if let Some((idx, _)) = new_watch {
                clause.watches[if update_first { 0 } else { 1 }] = idx;
                clause.watches.sort();
                changes.push((*clause_id, clause.literals[idx]));
            } else {
                // there is no new watch, so the clause is now unit or in conflict
                let unit_lit = clause.literals.iter().find(|lit| {
                    let id = lit.id();
                    self.vars[id].is_none()
                });
                if let Some(lit) = unit_lit {
                    self.stats.num_propagations += 1;
                    self.unit_literals.push_back(*lit);
                } else {
                    self.stats.num_conflicts += 1;
                    self.in_conflict = true;
                    self.unit_literals.clear();
                }
            }
        }

        // update watches
        for (clause_id, new_watch) in changes {
            if lit.positive() {
                self.var_watches[lit.id()].neg.retain(|id| *id != clause_id);
            } else {
                self.var_watches[lit.id()].pos.retain(|id| *id != clause_id);
            }
            if new_watch.positive() {
                self.var_watches[new_watch.id()].pos.push(clause_id);
            } else {
                self.var_watches[new_watch.id()].neg.push(clause_id);
            }
        }
    }

    pub fn unassign(&mut self, lit: Literal) {
        self.vars[lit.id()] = None;
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
        assert_eq!(state.var_watches.len(), 4);
    }

    #[test]
    fn test_state_assign() {
        let clauses = vec![Clause::from("1 2 3"), Clause::from("-1 -2 3")];
        let mut state = State::init(clauses);
        state.assign(Literal::from(1));
        assert_eq!(state.vars[1], Some(true));
        assert_eq!(state.clauses[0].watches, [0, 1]);
        assert_eq!(state.clauses[1].watches, [1, 2]);
        state.assign(Literal::from(2));
        assert_eq!(state.vars[2], Some(true));
        assert_eq!(state.clauses[1].watches, [1, 2]);
        println!("{:?}", state);
        assert_eq!(state.unit_literals[0], Literal::from(3));
        state.assign(Literal::from(-3));
        assert!(state.in_conflict);
    }
}
