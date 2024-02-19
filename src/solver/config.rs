use crate::solver::heuristic::HeuristicType;

pub struct Config {
    pub preprocessing: bool,
    pub pure_literal_elimination: bool,
    pub unit_propagation: bool,
    pub heuristic: HeuristicType,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            preprocessing: true,
            pure_literal_elimination: true,
            unit_propagation: true,
            heuristic: HeuristicType::Decay,
        }
    }
}
