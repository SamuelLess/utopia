use crate::cnf::{Clause, Literal, VarId};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct Preprocessor {
    mapping: HashMap<Literal, Literal>,
    units: Vec<Literal>,
}

impl Preprocessor {
    pub fn process(&mut self, cnf: Vec<Clause>) -> Option<Vec<Clause>> {
        let new_cnf = Preprocessor::remove_true_clauses(cnf);
        let units = Preprocessor::unit_propagation(new_cnf.clone())?;
        self.units = units.clone();
        let mut id = 1;
        let mut prop_cnf = Vec::new();
        for clause in new_cnf {
            // skip if satisfied
            if clause.literals.iter().any(|lit| units.contains(lit)) {
                continue;
            }
            let mut new_clause = vec![];
            for lit in clause.literals {
                // skip false literals
                if units.contains(&(-lit)) {
                    continue;
                }
                if !self.mapping.contains_key(&lit) {
                    let new_id = id as VarId;
                    self.mapping.insert(lit, Literal::from_value(new_id, true));
                    self.mapping.insert(
                        Literal::from_value(lit.id(), lit.negative()),
                        Literal::from_value(new_id, false),
                    );
                    id += 1;
                }
                let new_lit = *self.mapping.get(&lit).unwrap();
                new_clause.push(new_lit);
            }
            if new_clause.is_empty() {
                return None;
            }
            debug_assert_ne!(new_clause.len(), 1);
            prop_cnf.push(Clause::from(new_clause));
        }
        Some(prop_cnf)
    }

    pub fn map_solution(&self, solution: HashMap<VarId, bool>) -> HashMap<VarId, bool> {
        let mut backmap: HashMap<Literal, Literal> = HashMap::new();
        for (lit_from, lit_to) in &self.mapping {
            backmap.insert(*lit_to, *lit_from);
        }
        let mut real_solution: HashMap<VarId, bool> = solution
            .iter()
            .map(|(var_id, value)| {
                let lit = Literal::from_value(*var_id, *value);
                let back_lit = *backmap.get(&lit).unwrap();
                (back_lit.id(), back_lit.positive())
            })
            .collect();
        real_solution.extend(self.units.iter().map(|lit| (lit.id(), lit.positive())));
        real_solution
    }

    /// Returns all literals that are forced by unit propagation
    fn unit_propagation(cnf: Vec<Clause>) -> Option<Vec<Literal>> {
        let mut units: HashSet<Literal> = HashSet::new();
        let mut new_cnf = cnf.clone();
        let mut new_units = Preprocessor::get_units(new_cnf.clone());
        while !new_units.is_empty() {
            units.extend(new_units.clone());
            for lit in new_units.iter() {
                if units.contains(&(-*lit)) {
                    return None;
                }
            }
            new_cnf = Preprocessor::propagate(new_cnf.clone(), new_units.clone());
            if new_cnf.iter().any(|clause| clause.literals.is_empty()) {
                return None;
            }
            new_units = Preprocessor::get_units(new_cnf.clone());
        }
        Some(units.into_iter().collect())
    }

    fn get_units(cnf: Vec<Clause>) -> HashSet<Literal> {
        cnf.iter()
            .filter(|clause| clause.literals.len() == 1)
            .map(|clause| clause.literals[0])
            .collect()
    }

    /// Removes all true clauses and literals that are forced by unit propagation
    /// Empty clauses in the output imply UNSAT
    fn propagate(cnf: Vec<Clause>, units: HashSet<Literal>) -> Vec<Clause> {
        cnf.iter()
            .filter(|clause| clause.literals.iter().any(|lit| !units.contains(lit)))
            .map(|clause| {
                let new_literals: Vec<Literal> = clause
                    .literals
                    .iter()
                    .filter(|lit| !units.contains(&-**lit))
                    .cloned()
                    .collect();
                Clause::from(new_literals)
            })
            .collect()
    }

    pub fn remove_true_clauses(cnf: Vec<Clause>) -> Vec<Clause> {
        cnf.into_iter()
            .filter(|clause| {
                !clause
                    .literals
                    .iter()
                    .any(|lit| clause.literals.contains(&-*lit))
            })
            .collect()
    }
}
