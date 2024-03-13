use crate::solver::heuristic::HeuristicType;
use crate::solver::restarts::RestartPolicy;

pub struct Config {
    pub heuristic: HeuristicType,
    pub restart_policy: RestartPolicy,
    pub proof_file: Option<String>,
}

impl Config {
    pub fn new(
        heuristic: HeuristicType,
        proof_file: Option<String>,
        restart_policy: RestartPolicy,
    ) -> Self {
        Config {
            heuristic,
            proof_file,
            restart_policy,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            heuristic: HeuristicType::Decay,
            proof_file: None,
            restart_policy: RestartPolicy::Luby,
        }
    }
}
