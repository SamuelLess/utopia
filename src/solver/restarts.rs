use crate::solver::ema_policy::EMAPolicy;
use clap::ValueEnum;

const FIXED_INTERVAL_SIZE: usize = 700;
const GEOMETRIC_INTERVAL_SIZE: usize = 100;
const GEOMETRIC_MAGNIFICATION_FACTOR: f64 = 1.5;

#[derive(Debug, Clone)]
pub struct Restarter {
    num_restarts: usize,
    conflicts_since_last_restart: usize,
    restart_policy: RestartPolicy,
    ema_policy: Option<EMAPolicy>,
}

#[derive(Debug, Copy, Clone, ValueEnum, Eq, PartialEq)]
pub enum RestartPolicy {
    #[clap(name = "fixed-interval")]
    FixedInterval,
    #[clap(name = "geometric")]
    Geometric,
    #[clap(name = "luby")]
    Luby,
    #[clap(name = "glucose-ema")]
    GlucoseEma,
    #[clap(name = "no-restarts")]
    NoRestarts,
}

impl Restarter {
    pub fn init(restart_policy: RestartPolicy) -> Self {
        Restarter {
            num_restarts: 0,
            conflicts_since_last_restart: 0,
            restart_policy,
            ema_policy: match restart_policy {
                RestartPolicy::GlucoseEma => Some(EMAPolicy::init()),
                _ => None,
            },
        }
    }

    pub fn conflict(&mut self, learned_clause_lbd: usize, num_current_assignments: usize) {
        self.conflicts_since_last_restart += 1;

        if self.restart_policy == RestartPolicy::GlucoseEma {
            self.ema_policy
                .as_mut()
                .unwrap()
                .conflict(learned_clause_lbd, num_current_assignments);
        }
    }

    pub fn check_if_restart_necessary(&mut self) -> bool {
        let restart_necessary = match self.restart_policy {
            RestartPolicy::FixedInterval => self.fixed_interval_check_necessary(),
            RestartPolicy::Geometric => self.geometric_check_necessary(),
            RestartPolicy::Luby => self.luby_check_necessary(),
            RestartPolicy::GlucoseEma => self.lbd_ema_check_necessary(),
            RestartPolicy::NoRestarts => false,
        };

        if restart_necessary {
            self.conflicts_since_last_restart = 0;
            self.num_restarts += 1;
        }
        restart_necessary
    }

    fn fixed_interval_check_necessary(&mut self) -> bool {
        self.conflicts_since_last_restart >= FIXED_INTERVAL_SIZE
    }

    fn geometric_check_necessary(&mut self) -> bool {
        (self.conflicts_since_last_restart as f64)
            >= (GEOMETRIC_INTERVAL_SIZE as f64
                * (GEOMETRIC_MAGNIFICATION_FACTOR.powi(self.num_restarts as i32)))
    }

    fn luby_check_necessary(&mut self) -> bool {
        // luby sequence defined for i >= 1, but num_restarts >= 0 --> num_restarts + 1
        self.conflicts_since_last_restart >= 32 * Restarter::luby(self.num_restarts + 1)
    }

    fn luby(i: usize) -> usize {
        // don't store any variables inside of luby() calls -> otherwise stack overflow
        for k in 1..32 {
            if i == (1 << k) - 1 {
                return 1 << (k - 1);
            }
        }

        let mut k = 1;

        loop {
            if (1 << (k - 1)) <= i && i < (1 << k) - 1 {
                return Restarter::luby(i - (1 << (k - 1)) + 1);
            }
            k += 1;
        }
    }
    fn lbd_ema_check_necessary(&self) -> bool {
        match &self.ema_policy {
            Some(ema_policy) => {
                ema_policy.check_if_restart_necessary(self.conflicts_since_last_restart)
            }
            None => {
                panic!("EMA policy is not initialised")
            }
        }
    }
}
