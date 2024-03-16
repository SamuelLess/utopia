use crate::cnf::{ClauseId, Literal, VarId};
use crate::solver::clause_database::ClauseDatabase;
use crate::solver::literal_watching::LiteralWatcher;
use crate::solver::trail::Trail;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

pub struct Inprocessor {
    conflict_count: usize,
}

impl Inprocessor {
    pub fn init() -> Self {
        Inprocessor {
            conflict_count: 10000,
        }
    }

    pub fn inprocess(
        &mut self,
        clause_database: &mut ClauseDatabase,
        literal_watcher: &mut LiteralWatcher,
        trail: &Trail,
    ) {
        if self.conflict_count < 6000 {
            self.conflict_count += 1;
            return;
        }
        self.conflict_count = 0;
        println!("Inprocessing");
        // don't look at this code -- serious risk of brain damage :D
        let occ = clause_database
            .iter()
            .flat_map(|clause_id| {
                clause_database[clause_id]
                    .clone()
                    .map(move |x| (x, clause_id))
            })
            .into_group_map();

        let vars = occ.keys().map(|x| x.id()).collect::<HashSet<VarId>>();

        for var in vars {
            self.bounded_variable_elimination(var, clause_database, literal_watcher, trail, &occ);
        }
    }

    fn bounded_variable_elimination(
        &mut self,
        var_id: VarId,
        clause_database: &mut ClauseDatabase,
        literal_watcher: &mut LiteralWatcher,
        trail: &Trail,
        occ: &HashMap<Literal, Vec<ClauseId>>,
    ) {
        let mut new_clauses = vec![];

        let empty_vec = vec![];

        let pos_occ = occ
            .get(&Literal::from_value(var_id, true))
            .unwrap_or(&empty_vec);
        let neg_occ = occ
            .get(&Literal::from_value(var_id, false))
            .unwrap_or(&empty_vec);

        let num_clauses_before = pos_occ.len() + neg_occ.len();

        let pairs = pos_occ.iter().cartesian_product(neg_occ.iter());

        for (clause_1, clause_2) in pairs {
            let c1_iter = clause_database[*clause_1].literals.iter();
            let c2_iter = clause_database[*clause_2].literals.iter();

            let new_clause = c1_iter.chain(c2_iter).filter(|lit| lit.id() != var_id);

            // deduplicate new_clause
            let unique = new_clause.unique().collect_vec();

            // check for tautology
            if unique.len() == unique.iter().map(|lit| lit.id()).unique().count() {
                new_clauses.push(unique);
            }

            if new_clauses.len() >= num_clauses_before {
                return; // This won't be worthwhile. Abort and don't execute resolution.
            }
        }

        let num_added_clauses = new_clauses.len();

        assert!(num_added_clauses <= num_clauses_before);

        // Delete old clauses
        for clause_id in pos_occ.iter().chain(neg_occ.iter()) {
            clause_database.delete_clause_if_allowed(*clause_id, literal_watcher, trail);
        }

        println!(
            "Resolved {num_clauses_before} clauses for {}",
            num_added_clauses
        )
    }
}
