use crate::cnf::{Clause, ClauseId};
use crate::solver::literal_watching::LiteralWatcher;
use crate::solver::trail::{AssignmentReason, Trail};
use crate::solver::unit_propagation::UnitPropagator;
use itertools::Itertools;
use std::fmt::{Debug, Display, Formatter, Write};
use std::ops::Index;
use std::ops::IndexMut;
use std::ptr::write;

#[derive(Clone)]
pub struct ClauseDatabase {
    clauses: Vec<Clause>,
    first_learned_clause_id: ClauseId,
    free_clause_ids: Vec<ClauseId>,
    num_deletions: usize,
    conflicts_since_last_deletion: usize,
}

impl Debug for ClauseDatabase {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ClauseDatabase:")?;
        for clause in &self.clauses {
            writeln!(f, "LBD: {:?} {:?} ", clause.lbd, clause.literals)?;
        }
        writeln!(f, "")
    }
}
pub struct Iter<'a> {
    pos: i32,
    length: usize,
    free_clause_ids: &'a Vec<ClauseId>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = ClauseId;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.pos += 1;
            if self.pos >= self.length as i32 {
                return None;
            }
            if self
                .free_clause_ids
                .binary_search(&(self.pos as ClauseId))
                .is_err()
            {
                break;
            }
        }

        Some(self.pos as ClauseId)
    }
}
impl ClauseDatabase {
    pub fn init(clauses: Vec<Clause>) -> Self {
        ClauseDatabase {
            free_clause_ids: vec![],
            first_learned_clause_id: clauses.len(),
            clauses,
            num_deletions: 0,
            conflicts_since_last_deletion: 0,
        }
    }

    pub fn cnf(&self) -> &[Clause] {
        &self.clauses[0..self.first_learned_clause_id]
    }

    pub fn add_clause(&mut self, clause: Clause, literal_watcher: &mut LiteralWatcher) -> ClauseId {
        let id = if !self.free_clause_ids.is_empty() {
            let id = self.free_clause_ids.pop().unwrap();
            self.clauses[id] = clause;
            id
        } else {
            self.clauses.push(clause);
            self.clauses.len() - 1
        };

        literal_watcher.add_clause(&self.clauses[id], id);

        id
    }

    pub fn iter(&self) -> Iter {
        Iter {
            length: self.clauses.len(),
            pos: -1,
            free_clause_ids: &self.free_clause_ids,
        }
    }

    pub fn delete_clause_if_allowed(
        &mut self,
        clause_id: ClauseId,
        literal_watcher: &mut LiteralWatcher,
        trail: &Trail,
    ) {
        // Clauses that are currently reason clauses may NOT be removed
        let is_reason = trail
            .assignment_stack
            .iter()
            .any(|assignment| assignment.reason == AssignmentReason::Forced(clause_id));
        if is_reason {
            return;
        }

        if self.free_clause_ids.contains(&clause_id) {
            return;
        }

        // don't delete unit clauses
        if self.clauses[clause_id].literals.len() < 2 {
            return;
        }

        literal_watcher.delete_clause(&self.clauses[clause_id], clause_id);
        self.free_clause_ids.push(clause_id);
        self.free_clause_ids.sort_unstable();
    }

    pub fn delete_clauses_if_necessary(
        &mut self,
        literal_watcher: &mut LiteralWatcher,
        trail: &Trail,
    ) {
        if self.conflicts_since_last_deletion < 2000 + 300 * self.num_deletions {
            self.conflicts_since_last_deletion += 1;
            return;
        }

        self.conflicts_since_last_deletion = 0;
        self.num_deletions += 1;
        println!("Deleting clauses");

        let mut lbds = self
            .clauses
            .iter()
            .enumerate()
            .filter_map(|(clause_id, clause)| {
                if self.free_clause_ids.contains(&clause_id) {
                    None
                } else {
                    clause.lbd
                }
            })
            .collect_vec();

        lbds.sort();
        let threshold = lbds[lbds.len() / 2];

        for clause_id in 0..self.clauses.len() {
            if let Some(lbd) = self.clauses[clause_id].lbd {
                if lbd <= threshold {
                    continue;
                }
                // clauses with lbd = 2 ("glue clauses") should NOT be removed
                if self.clauses[clause_id].lbd.is_some()
                    && self.clauses[clause_id].lbd.unwrap() <= 2
                {
                    return;
                }

                self.delete_clause_if_allowed(clause_id, literal_watcher, trail);
            }
        }
        self.free_clause_ids.sort();
    }
}

impl Index<ClauseId> for ClauseDatabase {
    type Output = Clause;

    fn index(&self, index: ClauseId) -> &Self::Output {
        &self.clauses[index]
    }
}

impl IndexMut<ClauseId> for ClauseDatabase {
    fn index_mut(&mut self, index: ClauseId) -> &mut Self::Output {
        &mut self.clauses[index]
    }
}
