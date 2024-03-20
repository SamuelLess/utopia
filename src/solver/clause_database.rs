use crate::cnf::{Clause, ClauseId};
use crate::solver::literal_watching::LiteralWatcher;
use crate::solver::trail::{AssignmentReason, Trail};
use itertools::Itertools;
use std::cmp::max;
use std::fmt::{Debug, Formatter};
use std::ops::Index;
use std::ops::IndexMut;
use crate::solver::proof_logger::ProofLogger;

#[derive(Clone)]
pub struct ClauseDatabase {
    clauses: Vec<Clause>,
    free_clause_ids: Vec<ClauseId>,
    num_deletions: usize,
    pub(crate) proof_logger: ProofLogger,
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
    clause_database: &'a ClauseDatabase,
    necessary_clauses_only: bool,
    next_hole: Option<ClauseId>,
    next_hole_position: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = ClauseId;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.pos += 1;
            if self.pos >= self.clause_database.clauses.len() as i32 {
                return None;
            }

            if Some(self.pos as ClauseId) == self.next_hole {
                self.next_hole_position += 1;
                self.next_hole = self
                    .clause_database
                    .free_clause_ids
                    .get(self.next_hole_position)
                    .copied();
                continue;
            }

            if self.necessary_clauses_only
                && self.clause_database.clauses[self.pos as usize]
                    .lbd
                    .is_some()
            {
                continue;
            }

            break;
        }
        Some(self.pos as ClauseId)
    }
}
impl ClauseDatabase {
    pub fn init(clauses: &[Clause], proof_logging :bool) -> Self {
        ClauseDatabase {
            free_clause_ids: Vec::new(),
            clauses: clauses.to_vec(),
            num_deletions: 0,
            conflicts_since_last_deletion: 0,
            proof_logger: ProofLogger::new(proof_logging),
        }
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
        
        self.proof_logger.log(&self.clauses[id]);
        literal_watcher.add_clause(&self.clauses[id], id);

        id
    }

    pub fn iter(&self) -> Iter {
        Iter {
            pos: -1,
            clause_database: self,
            necessary_clauses_only: false,
            next_hole: self.free_clause_ids.first().copied(),
            next_hole_position: 0,
        }
    }
    pub fn necessary_clauses_iter(&self) -> Iter {
        Iter {
            pos: -1,
            clause_database: self,
            necessary_clauses_only: true,
            next_hole: self.free_clause_ids.first().copied(),
            next_hole_position: 0,
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
        
        self.proof_logger.delete(&self.clauses[clause_id]);
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
            .iter()
            .filter_map(|clause_id| self[clause_id].lbd)
            .collect_vec();

        lbds.sort();

        // don't delete glue clauses (lbd == 2)
        let threshold = max(lbds[lbds.len() / 2], 2);

        for clause_id in 0..self.clauses.len() {
            if let Some(lbd) = self.clauses[clause_id].lbd {
                if lbd <= threshold {
                    continue;
                }

                self.delete_clause_if_allowed(clause_id, literal_watcher, trail);
            }
        }
    }
}

impl Index<ClauseId> for ClauseDatabase {
    type Output = Clause;

    fn index(&self, index: ClauseId) -> &Self::Output {
        debug_assert!(
            !self.free_clause_ids.contains(&index),
            "Accessing deleted clause"
        );
        &self.clauses[index]
    }
}

impl IndexMut<ClauseId> for ClauseDatabase {
    fn index_mut(&mut self, index: ClauseId) -> &mut Self::Output {
        debug_assert!(
            !self.free_clause_ids.contains(&index),
            "Accessing deleted clause"
        );
        &mut self.clauses[index]
    }
}
