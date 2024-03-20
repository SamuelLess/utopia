use crate::cnf::Clause;
use std::io::Write;


#[derive(Debug, Clone)]
pub enum ProofStep {
    AddClause(Clause),
    DeleteClause(Clause),
}
#[derive(Debug, Clone, Default)]
pub struct ProofLogger {
    pub active: bool,
    pub proof: Vec<ProofStep>,
}

// TODO: the file should already be written during the search to avoid the log file
//       filling up the memory (when we start deleting clauses)

impl ProofLogger {
    pub fn new(active: bool) -> Self {
        ProofLogger {
            proof: vec![],
            active,
        }
    }

    pub fn log(&mut self, clause: &Clause) {
        if !self.active {
            return;
        }

        self.proof.push(ProofStep::AddClause(clause.clone()));
    }
    
    pub fn delete(&mut self, clause: &Clause) {
        if !self.active {
            return;
        }

        self.proof.push(ProofStep::DeleteClause(clause.clone()));
    }

    pub fn write_to_file(&self, filename: &str) {
        let mut file = std::fs::File::create(filename).unwrap();
        for proof_step in &self.proof {
            
            
            let clause = match proof_step {
                ProofStep::AddClause(clause) => clause,
                ProofStep::DeleteClause(clause) => clause,
            };
            
            let clause_str = clause
                .literals
                .iter()
                .map(|lit| format!("{}", lit))
                .collect::<Vec<String>>()
                .join(" ");
            
            match proof_step {
                ProofStep::AddClause(_) => {}
                ProofStep::DeleteClause(_) => {write!(file, "d ").unwrap()}
            }
            
            writeln!(file, "{} 0", clause_str).unwrap();
        }
    }
}
