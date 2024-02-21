use crate::solver::heuristic::HeuristicType;

pub struct Config {
    pub heuristic: HeuristicType,
}

impl Config {
    pub fn new(heuristic: HeuristicType) -> Self {
        Config { heuristic }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            heuristic: HeuristicType::Decay,
        }
    }
}
