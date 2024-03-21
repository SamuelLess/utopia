use fnv::FnvHasher;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::hash::BuildHasherDefault;
use std::ops::Neg;
use std::str::FromStr;

type FastHasher = BuildHasherDefault<FnvHasher>;
use crate::solver::trail::{Assignment, Trail};

pub fn check_assignment(clauses: &[Clause], assignment: HashMap<VarId, bool>) -> bool {
    clauses.iter().all(|clause| {
        clause.clone().any(|lit| {
            if let Some(assignment_value) = assignment.get(&(lit.id())) {
                (lit.positive()) == *assignment_value
            } else {
                false
            }
        })
    })
}

pub type VarId = usize;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Literal {
    value: i32,
}

impl Literal {
    pub fn new(value: i32) -> Self {
        Literal { value }
    }

    pub fn from_value(id: VarId, positive: bool) -> Self {
        Literal {
            value: if positive { id as i32 } else { -(id as i32) },
        }
    }

    pub fn id(&self) -> VarId {
        self.value.unsigned_abs() as VarId
    }

    pub fn positive(&self) -> bool {
        self.value > 0
    }
    pub fn negative(&self) -> bool {
        self.value < 0
    }

    pub fn id_val(&self) -> (VarId, bool) {
        (self.id(), self.positive())
    }

    pub fn is_true(&self, vars: &[Option<bool>]) -> bool {
        vars[self.id()] == Some(self.positive())
    }

    pub fn is_false(&self, vars: &[Option<bool>]) -> bool {
        vars[self.id()] == Some(self.negative())
    }

    pub fn non_false(&self, vars: &[Option<bool>]) -> bool {
        vars[self.id()] != Some(self.negative())
    }

    pub fn is_free(&self, vars: &[Option<bool>]) -> bool {
        vars[self.id()].is_none()
    }

    pub fn value(&self, vars: &[Option<bool>]) -> Option<bool> {
        if self.is_free(vars) {
            None
        } else {
            Some(self.is_true(vars))
        }
    }
}

impl FromStr for Literal {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse().map_err(|_| ())?;
        Ok(Literal::new(value))
    }
}

impl From<i32> for Literal {
    fn from(value: i32) -> Self {
        Literal::new(value)
    }
}

impl From<Assignment> for Literal {
    fn from(assignment: Assignment) -> Self {
        assignment.literal
    }
}

impl Neg for Literal {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Literal::new(-self.value)
    }
}

impl Debug for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub type ClauseId = usize;

#[derive(Debug, Clone, PartialEq)]
pub struct Clause {
    pub literals: Vec<Literal>,
    pub blocking_literal: Literal,
    pub lbd: Option<usize>,
}

impl Clause {
    pub fn from_literals_and_lbd(literals: Vec<Literal>, lbd: usize) -> Self {
        Clause {
            blocking_literal: *literals.first().unwrap_or(&Literal::new(0)),
            literals,
            lbd: Some(lbd),
        }
    }

    pub fn update_lbd(&mut self, trail: &mut Trail) {
        if let Some(old_lbd) = self.lbd {
            let new_lbd = self
                .literals
                .iter()
                .map(|lit| trail.var_decision_level[lit.id()])
                .collect::<HashSet<_, FastHasher>>()
                .len();

            if new_lbd < old_lbd {
                self.lbd = Some(new_lbd);
            }
        }
    }

    pub fn is_satisfied(&self, vars: &[Option<bool>]) -> bool {
        if self.check_blocking_literal(vars) {
            return true;
        }
        self.literals.iter().any(|lit| lit.is_true(vars))
    }

    pub fn is_conflict(&self, vars: &[Option<bool>]) -> bool {
        self.literals.iter().all(|lit| lit.is_false(vars))
    }

    pub fn check_blocking_literal(&self, vars: &[Option<bool>]) -> bool {
        self.blocking_literal.is_true(vars)
    }

    pub fn resolution(self, other: Self) -> Self {
        let mut new_literals = self.literals.clone();
        new_literals.extend(other.literals);
        new_literals.sort_unstable();
        new_literals.dedup();
        let to_filter = new_literals
            .clone()
            .into_iter()
            .filter(|lit| new_literals.contains(&-*lit))
            .collect::<Vec<_>>();
        new_literals.retain(|lit| !to_filter.contains(lit));
        Clause::from(new_literals)
    }
}

impl From<Vec<Literal>> for Clause {
    fn from(literals: Vec<Literal>) -> Self {
        Clause {
            blocking_literal: *literals.first().unwrap_or(&Literal::new(0)),
            literals,
            lbd: None,
        }
    }
}

impl From<&str> for Clause {
    fn from(s: &str) -> Self {
        let literals: Vec<Literal> = s
            .split_whitespace()
            .map(|lit| lit.parse().unwrap())
            .collect();
        Clause::from(literals)
    }
}

impl Iterator for Clause {
    type Item = Literal;

    fn next(&mut self) -> Option<Self::Item> {
        self.literals.pop()
    }
}

impl Display for Clause {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.literals)
    }
}

pub type SolutionAssignment = HashMap<VarId, bool>;
pub type Solution = Option<SolutionAssignment>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clause_resolution() {
        let clause1 = Clause::from("1 2 3");
        let clause2 = Clause::from("1 2 -3");
        assert_eq!(clause1.resolution(clause2), Clause::from("1 2"));
        let clause1 = Clause::from("1 2 3");
        let clause2 = Clause::from("1 2 3");
        assert_eq!(clause1.resolution(clause2), Clause::from("1 2 3"));
        let clause1 = Clause::from("1 2 3");
        let clause2 = Clause::from("1 -2 -3");
        assert_eq!(clause1.resolution(clause2), Clause::from("1"));
        let clause1 = Clause::from("1 2 3");
        let clause2 = Clause::from("-2 -3 4");
        assert_eq!(clause1.resolution(clause2), Clause::from("1 4"));
    }
}
