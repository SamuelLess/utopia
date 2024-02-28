use crate::cnf::Clause;
use std::io::Write;

#[derive(Debug, Clone, Default)]
pub struct ProofLogger {
    pub active: bool,
    pub proof: Vec<Clause>,
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

        self.proof.push(clause.clone());
    }

    pub fn write_to_file(&self, filename: &str) {
        let mut file = std::fs::File::create(filename).unwrap();
        for clause in &self.proof {
            let clause_str = clause
                .literals
                .iter()
                .map(|lit| format!("{}", lit))
                .collect::<Vec<String>>()
                .join(" ");
            writeln!(file, "{} 0", clause_str).unwrap();
        }
    }
}
