use crate::solver::branching::Assignment;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Neg;
use std::str::FromStr;

pub fn check_assignment(clauses: Vec<Clause>, assignment: HashMap<VarId, bool>) -> bool {
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
        Literal::from_value(assignment.var, assignment.value)
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

#[derive(Debug, Clone)]
pub struct Clause {
    pub literals: Vec<Literal>,
    pub watches: [usize; 2],
}

impl Clause {
    pub fn is_satisfied(&self, vars: &[Option<bool>]) -> bool {
        self.literals
            .iter()
            .any(|lit| vars[lit.id()] == Some(lit.positive()))
    }

    pub fn watches(&self) -> [Literal; 2] {
        [
            self.literals[self.watches[0]],
            self.literals[self.watches[1]],
        ]
    }

    /// Returns all indices with non-false entries.
    pub fn possible_watches_idx(&self, vars: &[Option<bool>]) -> Vec<usize> {
        self.literals
            .iter()
            .enumerate()
            .filter(|(_, lit)| vars[lit.id()] != Some(!lit.positive()))
            .map(|(i, _)| i)
            .collect()
    }
}

impl From<Vec<Literal>> for Clause {
    fn from(literals: Vec<Literal>) -> Self {
        Clause {
            literals,
            watches: [0, 1],
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
