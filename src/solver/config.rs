use crate::solver::heuristic::HeuristicType;
use crate::solver::progress::ProgressPrintingInterval;
use crate::solver::restarts::RestartPolicy;

pub struct Config {
    pub heuristic: HeuristicType,
    pub restart_policy: RestartPolicy,
    pub proof_file: Option<String>,
    pub inprocessing: bool,
    pub progress_printing_interval: ProgressPrintingInterval,
}

impl Config {
    pub fn new(
        heuristic: HeuristicType,
        proof_file: Option<String>,
        restart_policy: RestartPolicy,
        inprocessing: bool,
        progress_printing_interval: ProgressPrintingInterval,
    ) -> Self {
        Config {
            heuristic,
            proof_file,
            restart_policy,
            inprocessing,
            progress_printing_interval,
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
            progress_printing_interval: ProgressPrintingInterval::Medium,
        }
    }
}
