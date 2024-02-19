use crate::cnf::{Clause, Literal, VarId};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct Preprocessor {
    mapping: HashMap<Literal, Literal>,
}

impl Preprocessor {
    pub fn process(&mut self, cnf: Vec<Clause>) -> Vec<Clause> {
        let units = Preprocessor::unit_propagation(cnf.clone());
        let mut id = 1;
        let mut new_cnf: Vec<Clause> = Vec::new();
        for clause in cnf {
            if clause.literals.iter().any(|lit| units.contains(lit)) {
                continue;
            }
            let mut new_clause = vec![];
            for lit in clause.literals {
                if units.contains(&lit) {
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
            assert_ne!(new_clause.len(), 1);
            new_cnf.push(Clause::from(new_clause));
        }
        new_cnf
    }

    pub fn map_solution(&self, solution: HashMap<VarId, bool>) -> HashMap<VarId, bool> {
        let mut backmap: HashMap<Literal, Literal> = HashMap::new();
        for (lit_from, lit_to) in &self.mapping {
            backmap.insert(*lit_to, *lit_from);
        }
        solution
            .iter()
            .map(|(var_id, value)| {
                let lit = Literal::from_value(*var_id, *value);
                let back_lit = *backmap.get(&lit).unwrap();
                (back_lit.id(), back_lit.positive())
            })
            .collect()
    }

    /// Returns all literals that are forced by unit propagation
    fn unit_propagation(cnf: Vec<Clause>) -> Vec<Literal> {
        let mut units: HashSet<Literal> = Preprocessor::get_units(cnf.clone());
        let mut new_cnf = Preprocessor::propagate(cnf.clone(), units.clone());
        let mut new_units = Preprocessor::get_units(new_cnf.clone());
        while !new_units.is_empty() {
            units.extend(new_units.clone());
            new_cnf = Preprocessor::propagate(new_cnf.clone(), new_units.clone());
            new_units = Preprocessor::get_units(new_cnf.clone());
        }
        units.into_iter().collect()
    }

    fn get_units(cnf: Vec<Clause>) -> HashSet<Literal> {
        cnf.iter()
            .filter(|clause| clause.literals.len() == 1)
            .map(|clause| clause.literals[0])
            .collect()
    }

    fn propagate(cnf: Vec<Clause>, units: HashSet<Literal>) -> Vec<Clause> {
        cnf.iter()
            .filter(|clause| clause.literals.iter().any(|lit| units.contains(lit)))
            .cloned()
            .collect()
    }
}
