use crate::solver::heuristic::HeuristicType;

pub struct Config {
    pub heuristic: HeuristicType,
    pub proof_file: Option<String>,
}

impl Config {
    pub fn new(heuristic: HeuristicType, proof_file: Option<String>) -> Self {
        Config {
            heuristic,
            proof_file,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            heuristic: HeuristicType::Decay,
            proof_file: None,
        }
    }
}
