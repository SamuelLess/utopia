const LBD_EMA_SHORT_TERM_ALPHA: f64 = 2.0 / 51.0; // Window size of 50

// As in Biere & Fröhlich, should be between 2e-12 and 2e-18
const LBD_EMA_LONG_TERM_ALPHA: f64 = 2e-6;
const ASSIGNMENT_EMA_SHORT_TERM_ALPHA: f64 = 2.0 / 51.0; // Window size of 50
const ASSIGNMENT_EMA_LONG_TERM_ALPHA: f64 = 2e-6;

const MARGIN_RATIO_FORCING_RESTART: f64 = 1.15;
const MARGIN_RATIO_BLOCKING_RESTART: f64 = 1.4;

#[derive(Debug, Clone)]
pub struct EMAPolicy {
    lbd_short_term: ExponentialMovingAverage,
    lbd_long_term: ExponentialMovingAverage,
    assignments_short_term: ExponentialMovingAverage,
    assignments_long_term: ExponentialMovingAverage,
}

impl EMAPolicy {
    pub fn init() -> Self {
        EMAPolicy {
            lbd_short_term: ExponentialMovingAverage::init(LBD_EMA_SHORT_TERM_ALPHA),
            lbd_long_term: ExponentialMovingAverage::init(LBD_EMA_LONG_TERM_ALPHA),
            assignments_short_term: ExponentialMovingAverage::init(ASSIGNMENT_EMA_SHORT_TERM_ALPHA),
            assignments_long_term: ExponentialMovingAverage::init(ASSIGNMENT_EMA_LONG_TERM_ALPHA),
        }
    }

    pub fn conflict(&mut self, learned_clause_lbd: usize, num_current_assignments: usize) {
        let learned_clause_lbd = learned_clause_lbd as f64;
        self.lbd_long_term.update(learned_clause_lbd);
        self.lbd_short_term.update(learned_clause_lbd);

        let num_current_assignments = num_current_assignments as f64;
        self.assignments_long_term.update(num_current_assignments);
        self.assignments_short_term.update(num_current_assignments);
        /*
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open("log.csv")
            .unwrap();

        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let record = &[time.to_string(), self.restart_necessary().to_string(), self.restart_blocked().to_string(), self.lbd_long_term.value.to_string(), self.lbd_short_term.value.to_string(), self.assignments_long_term.value.to_string(), self.assignments_short_term.value.to_string(), learned_clause_lbd.to_string(), num_current_assignments.to_string()];
        writeln!(file, "{}", record.join(",")).unwrap();
        file.flush().unwrap();*/
    }

    pub fn check_if_restart_necessary(&self, conflicts_since_last_restart: usize) -> bool {
        // open log.csv

        conflicts_since_last_restart >= 50 && self.restart_necessary() && !self.restart_blocked()
    }

    fn restart_necessary(&self) -> bool {
        self.lbd_short_term.value > MARGIN_RATIO_FORCING_RESTART * self.lbd_long_term.value
    }

    fn restart_blocked(&self) -> bool {
        self.assignments_short_term.value
            > MARGIN_RATIO_BLOCKING_RESTART * self.assignments_long_term.value
    }
}

/// Exponential Moving Average based on Glucose restart as described in
/// A. Biere and A. Fröhlich, “Evaluating CDCL Restart Schemes,” pp. 1--17. doi: 10.29007/89dw.
/// Uses their initialization scheme, starting with a bigger alpha until the target alpha is reached
#[derive(Debug, Clone)]
struct ExponentialMovingAverage {
    value: f64,
    alpha: f64,
    target_alpha: f64,
}

impl ExponentialMovingAverage {
    fn init(target_alpha: f64) -> Self {
        assert!(0.0 < target_alpha && target_alpha < 1.0);
        ExponentialMovingAverage {
            value: 1.0,
            alpha: 1.0,
            target_alpha,
        }
    }

    /// Adds value to the ema and returns the new EMA
    fn update(&mut self, new_value: f64) -> f64 {
        // Adjust alpha for initialization
        if self.alpha != self.target_alpha {
            self.alpha /= 1.02;
            if self.alpha < self.target_alpha {
                self.alpha = self.target_alpha;
            }
        }

        // EMA(n, α) := α · tn + (1 − α) · EMA(n − 1, α), with 0 < α <= 1
        self.value = self.alpha * new_value + (1.0 - self.alpha) * self.value;

        self.value
    }
}
