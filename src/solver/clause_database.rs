use crate::cnf::{Clause, ClauseId};
use crate::solver::literal_watching::LiteralWatcher;
use crate::solver::trail::{AssignmentReason, Trail};
use itertools::Itertools;
use std::ops::Index;
use std::ops::IndexMut;

#[derive(Debug, Clone)]
pub struct ClauseDatabase {
    clauses: Vec<Clause>,
    first_learnt_clause_id: ClauseId,
    free_clause_ids: Vec<ClauseId>,
    num_deletions: usize,
    conflicts_since_last_deletion: usize,
}

impl ClauseDatabase {
    pub fn init(clauses: Vec<Clause>) -> Self {
        ClauseDatabase {
            free_clause_ids: vec![],
            first_learnt_clause_id: clauses.len(),
            clauses,
            num_deletions: 0,
            conflicts_since_last_deletion: 0,
        }
    }

    pub fn cnf(&self) -> &[Clause] {
        &self.clauses[0..self.first_learnt_clause_id]
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

    pub fn delete_clauses_if_neccessary(
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

        for (clause_id, clause) in self.clauses.iter().enumerate() {
            if let Some(lbd) = clause.lbd {
                if lbd <= threshold {
                    continue;
                }

                // clauses with lbd = 2 ("glue clauses") should NOT be removed
                if lbd < 2 {
                    continue;
                }

                // Clauses that are currently reason clauses may NOT be removed
                let is_reason = trail
                    .assignment_stack
                    .iter()
                    .any(|assignment| assignment.reason == AssignmentReason::Forced(clause_id));
                if is_reason {
                    continue;
                }

                if self.free_clause_ids.contains(&clause_id) {
                    continue;
                }

                literal_watcher.delete_clause(clause, clause_id);
                self.free_clause_ids.push(clause_id);
            }
        }
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
