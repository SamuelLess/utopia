use crate::solver::statistics::StateStatistics;
use colored::{ColoredString, Colorize};

pub struct Progress {
    time_of_last_print: std::time::Instant,
    last_num_conflicts: usize,
    last_num_assignments: usize,
    last_num_clauses: usize,
    last_num_restarts: usize,
}

const PRINT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);

const TIME: usize = 6;
const CONFLICTS_TOTAL: usize = 15;
const CONFLICTS_DELTA: usize = 10;
const RESTARTS_TOTAL: usize = 12;
const RESTARTS_DELTA: usize = 9;
const ASSIGNMENTS_TOTAL: usize = 15;
const ASSIGNMENTS_CUR: usize = 10;
const ASSIGNMENTS_DELTA: usize = 8;
const CLAUSES_CUR: usize = 15;
const CLAUSES_DELTA: usize = 15;

impl Progress {
    pub fn new() -> Self {
        Self::print_header();
        Progress {
            time_of_last_print: std::time::Instant::now(),
            last_num_conflicts: 0,
            last_num_assignments: 0,
            last_num_clauses: 0,
            last_num_restarts: 0,
        }
    }

    pub fn print_progress_if_necessary(
        &mut self,
        state_statistics: &StateStatistics,
        current_num_assignments: usize,
        current_num_clauses: usize,
    ) {
        if self.time_of_last_print.elapsed() > PRINT_INTERVAL {
            self.print_progress(
                state_statistics,
                current_num_assignments,
                current_num_clauses,
            );
            self.time_of_last_print = std::time::Instant::now();
        }
    }

    fn print_header() {
        let conflicts = CONFLICTS_TOTAL + CONFLICTS_DELTA + 1;
        let restarts = RESTARTS_TOTAL + RESTARTS_DELTA + 1;
        let assignments = ASSIGNMENTS_TOTAL + ASSIGNMENTS_CUR + ASSIGNMENTS_DELTA + 2;
        let clauses = CLAUSES_CUR + CLAUSES_DELTA + 1;
        println!(
            "c ┌─{:─<TIME$}─┬─{:─<conflicts$}─┬─{:─<restarts$}─┬─{:─<assignments$}─┬─{:─<clauses$}─┐",
            "", "", "", "", ""
        );
        println!(
            "c │ {:<TIME$} │ {:<conflicts$} │ {:<restarts$} │ {:<assignments$} │ {:<clauses$} │",
            "Time", "Conflicts", "Restarts", "Assignments", "Clauses"
        );
        println!(
            "c │ {:<TIME$} │ {:>CONFLICTS_TOTAL$} {:<CONFLICTS_DELTA$} │ {:>RESTARTS_TOTAL$} {:<RESTARTS_DELTA$} │ {:>ASSIGNMENTS_TOTAL$} {:>ASSIGNMENTS_CUR$} {:>ASSIGNMENTS_DELTA$} │ {:<clauses$} │",
            "(sec.)".truecolor(100,100,100), "", "", "", "", "total".truecolor(100,100,100), "current".truecolor(100,100,100),"", "",
        );
        println!(
            "c │┈{:┈<TIME$}┈│┈{:┈<CONFLICTS_TOTAL$}┈{:┈<CONFLICTS_DELTA$}┈│┈{:┈<RESTARTS_TOTAL$}┈{:┈<RESTARTS_DELTA$}┈│┈{:┈<ASSIGNMENTS_TOTAL$}┈{:┈>ASSIGNMENTS_DELTA$}┈{:┈>ASSIGNMENTS_CUR$}┈│┈{:┈<clauses$}┈│",
            "", "", "", "", "", "", "", "", ""
        );
    }

    fn print_progress(
        &mut self,
        state_statistics: &StateStatistics,
        current_num_assignments: usize,
        current_num_clauses: usize,
    ) {
        println!(
            "c │ {:>TIME$} │ {:>CONFLICTS_TOTAL$} {:<CONFLICTS_DELTA$} │ {:>RESTARTS_TOTAL$} {:<RESTARTS_DELTA$} │ {:>ASSIGNMENTS_TOTAL$} {:>ASSIGNMENTS_CUR$} {:<ASSIGNMENTS_DELTA$} │ {:>CLAUSES_CUR$} {:<CLAUSES_DELTA$} │",
            state_statistics.start_time.elapsed().as_secs(),
            state_statistics.num_conflicts,
            Self::print_delta(self.last_num_conflicts as i32, state_statistics.num_conflicts as i32, false),
            state_statistics.num_restarts,
            Self::print_delta(self.last_num_restarts as i32, state_statistics.num_restarts as i32, false),
            state_statistics.num_assignments,
            current_num_assignments,
            Self::print_delta(self.last_num_assignments as i32, current_num_assignments as i32, true),
            current_num_clauses,
            Self::print_delta(self.last_num_clauses as i32, current_num_clauses as i32, true),
        );

        self.last_num_conflicts = state_statistics.num_conflicts;
        self.last_num_assignments = current_num_assignments;
        self.last_num_clauses = current_num_clauses;
        self.last_num_restarts = state_statistics.num_restarts;
    }

    pub fn close_table(&self) {
        let conflicts = CONFLICTS_TOTAL + CONFLICTS_DELTA + 1;
        let restarts = RESTARTS_TOTAL + RESTARTS_DELTA + 1;
        let assignments = ASSIGNMENTS_TOTAL + ASSIGNMENTS_CUR + ASSIGNMENTS_DELTA + 2;
        let clauses = CLAUSES_CUR + CLAUSES_DELTA + 1;
        println!(
            "c └─{:─<TIME$}─┴─{:─<conflicts$}─┴─{:─<restarts$}─┴─{:─<assignments$}─┴─{:─<clauses$}─┘",
            "", "", "", "", ""
        );
    }

    fn print_delta(old_value: i32, new_value: i32, use_colors: bool) -> ColoredString {
        let mut output = String::new();
        output.push('(');

        let delta = new_value - old_value;
        if delta >= 0 {
            output.push('+');
            output.push_str(&delta.to_string());
            output.push(')');
            if use_colors {
                return output.truecolor(0, 150, 0);
            }
        } else {
            output.push_str(&delta.to_string());
            output.push(')');
            if use_colors {
                return output.truecolor(150, 0, 0);
            }
        }
        output.truecolor(120, 120, 120)
    }
}
