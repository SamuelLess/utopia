use crate::solver::heuristic::HeuristicType;
use crate::solver::restarts::RestartPolicy;

pub struct Config {
    pub heuristic: HeuristicType,
    pub restart_policy: RestartPolicy,
    pub proof_file: Option<String>,
    pub inprocessing: bool,
}

impl Config {
    pub fn new(
        heuristic: HeuristicType,
        proof_file: Option<String>,
        restart_policy: RestartPolicy,
        inprocessing: bool,
    ) -> Self {
        Config {
            heuristic,
            proof_file,
            restart_policy,
            inprocessing,
        }
    }
}

impl Default for Config {
    // usually only used for tests
    fn default() -> Self {
        Config {
            heuristic: HeuristicType::VSIDS,
            proof_file: None,
            restart_policy: RestartPolicy::GlucoseEma,
            inprocessing: true,
        }
    }
}
