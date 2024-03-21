use crate::cnf::{Clause, ClauseId, Literal};
use std::ops::Neg;

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
    Satisfied(Literal),
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
        // unit clauses don't need watches
        if clause.literals.len() < 2 {
            return;
        }

        for lit in &clause.literals[0..2] {
            self.add_watch(*lit, clause_id);
        }
    }

    pub fn delete_clause(&mut self, clause: &Clause, clause_id: ClauseId) {
        for lit in &clause.literals[0..2] {
            self.affected_clauses(lit.neg())
                .retain(|&id| id != clause_id);
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
        invalid_literal: Literal,
        vars: &[Option<bool>],
    ) -> WatchUpdate {
        // ensure that the first watch is the newly invalid one
        if clause.literals[0].id() != invalid_literal.id() {
            clause.literals.swap(0, 1);
        }

        assert_eq!(clause.literals[0], invalid_literal);

        assert!(invalid_literal.is_false(vars));

        // the other literal is also invalid
        if clause.literals[1].is_false(vars) {
            return WatchUpdate::Conflict;
        }

        for i in 0..clause.literals.len() {
            if clause.literals[i].is_true(vars) {
                return WatchUpdate::Satisfied(clause.literals[i]);
            }

            // the first two literals can't become new watches as the already are
            if i > 1 && clause.literals[i].is_free(vars) {
                // new watch found -- swap it into the watch position
                clause.literals.swap(0, i);
                return WatchUpdate::FoundNewWatch;
            }
        }
        // verify that the clause is actually unit
        debug_assert_eq!(
            clause
                .literals
                .iter()
                .map(|lit| lit.value(vars))
                .filter(|v| v.is_none())
                .count(),
            1
        );
        debug_assert_eq!(
            clause
                .literals
                .iter()
                .map(|lit| lit.value(vars))
                .filter(|v| *v == Some(true))
                .count(),
            0
        );

        WatchUpdate::Unit(clause.literals[1])
    }

    fn create_watches(clauses: &[Clause], num_vars: usize) -> Vec<VarWatch> {
        let mut watches = vec![VarWatch::default(); num_vars + 1];
        for (clause_id, clause) in clauses.iter().enumerate() {
            if clause.literals.is_empty() {
                continue; // skip empty clauses
            }
            if clause.literals.len() == 1 {
                continue; // Don't watch unit clauses, they never change
            }

            for lit in &clause.literals[0..2] {
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
