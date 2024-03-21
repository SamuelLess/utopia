use crate::solver::statistics::StateStatistics;
use colored::{ColoredString, Colorize};

pub struct Progress {
    time_of_last_print: std::time::Instant,
    last_num_conflicts: usize,
    last_num_total_assignments: usize,
    last_num_cur_assignments: usize,
    last_assigned_vars_percent: usize,
    last_num_clauses: usize,
    last_num_restarts: usize,
    last_inprocessor_total_time: u128,
    last_inprocessor_resolved: usize,
}

const PRINT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);

const TIME: usize = 5;
const CONFLICTS_TOTAL: usize = 10;
const RESTARTS_TOTAL: usize = 8;
const ASSIGNMENTS_TOTAL: usize = 13;
const ASSIGNED_VARS_NUM: usize = 11;
const ASSIGNED_VARS_PERC: usize = 8;
const CLAUSES_CUR: usize = 12;
const INPROCESSOR_RESOLVED: usize = 11;
const INPROCESSOR_TIME: usize = 10;

impl Progress {
    pub fn new() -> Self {
        Self::print_header();
        Progress {
            time_of_last_print: std::time::Instant::now(),
            last_num_conflicts: 0,
            last_num_total_assignments: 0,
            last_num_cur_assignments: 0,
            last_assigned_vars_percent: 0,
            last_num_clauses: 0,
            last_num_restarts: 0,
            last_inprocessor_total_time: 0,
            last_inprocessor_resolved: 0,
        }
    }

    pub fn print_progress_if_necessary(
        &mut self,
        state_statistics: &StateStatistics,
        current_num_assignments: usize,
        current_num_clauses: usize,
        resolved_vars: usize,
        inprocessor_time: u128,
    ) {
        if self.time_of_last_print.elapsed() > PRINT_INTERVAL {
            self.print_progress(
                state_statistics,
                current_num_assignments,
                current_num_clauses,
                resolved_vars,
                inprocessor_time,
            );
            self.time_of_last_print = std::time::Instant::now();
        }
    }

    fn print_header() {
        println!(
            "c ┌─\
            {:─<TIME$}─┬─\
            {:─<CONFLICTS_TOTAL$}─┬─\
            {:─<RESTARTS_TOTAL$}─┬─\
            {:─<ASSIGNMENTS_TOTAL$}─{:─<ASSIGNED_VARS_NUM$}─{:─<ASSIGNED_VARS_PERC$}─┬─\
            {:─<CLAUSES_CUR$}─┬─\
            {:─<INPROCESSOR_RESOLVED$}─{:─<INPROCESSOR_TIME$}─┐",
            "", "", "", "", "", "", "", "", ""
        );
        println!(
            "c │ \
            {:<TIME$} │ \
            {:<CONFLICTS_TOTAL$} │ \
            {:<RESTARTS_TOTAL$} │ \
            {:<ASSIGNMENTS_TOTAL$} {:<ASSIGNED_VARS_NUM$} {:<ASSIGNED_VARS_PERC$} │ \
            {:<CLAUSES_CUR$} │ \
            {:<INPROCESSOR_RESOLVED$} {:<INPROCESSOR_TIME$} │",
            "Time", "Conflicts", "Restarts", "Assignments", "", "", "Clauses", "Inprocessor", ""
        );
        println!(
            "c │ \
            {:<TIME$} │ \
            {:>CONFLICTS_TOTAL$} │ \
            {:>RESTARTS_TOTAL$} │ \
            {:>ASSIGNMENTS_TOTAL$} {:>ASSIGNED_VARS_NUM$} {:<ASSIGNED_VARS_PERC$} │ \
            {:<CLAUSES_CUR$} │ \
            {:>INPROCESSOR_RESOLVED$} {:>INPROCESSOR_TIME$} │",
            "",
            "",
            "",
            "total".truecolor(100, 100, 100),
            "current".truecolor(100, 100, 100),
            "vars".truecolor(100, 100, 100),
            "",
            "resolved".truecolor(100, 100, 100),
            "time".truecolor(100, 100, 100)
        );
    }

    fn print_progress(
        &mut self,
        state_statistics: &StateStatistics,
        current_num_assignments: usize,
        current_num_clauses: usize,
        resolved_vars: usize,
        inprocessor_time_millis: u128,
    ) {
        if state_statistics.num_vars == 0 {
            return;
        }

        let assigned_vars_percent =
            (current_num_assignments as f64 / state_statistics.num_vars as f64 * 100.0).round()
                as usize;

        println!(
            "c │┈\
            {:┈<TIME$}┈│┈\
            {:┈<CONFLICTS_TOTAL$}┈│┈\
            {:┈<RESTARTS_TOTAL$}┈│┈\
            {:┈<ASSIGNMENTS_TOTAL$}┈{:┈>ASSIGNED_VARS_NUM$}┈{:┈>ASSIGNED_VARS_PERC$}┈│┈\
            {:┈<CLAUSES_CUR$}┈│┈\
            {:┈<INPROCESSOR_RESOLVED$}┈{:┈<INPROCESSOR_TIME$}┈│",
            "", "", "", "", "", "", "", "", ""
        );
        println!(
            "c │ \
            {:<TIME$} │ \
            {:>CONFLICTS_TOTAL$} │ \
            {:>RESTARTS_TOTAL$} │ \
            {:>ASSIGNMENTS_TOTAL$} {:>ASSIGNED_VARS_NUM$} {:<ASSIGNED_VARS_PERC$} │ \
            {:>CLAUSES_CUR$} │ \
            {:>INPROCESSOR_RESOLVED$} {:>INPROCESSOR_TIME$} │",
            state_statistics.start_time.elapsed().as_secs(),
            state_statistics.num_conflicts,
            state_statistics.num_restarts,
            state_statistics.num_assignments,
            current_num_assignments,
            format!("({}%)", assigned_vars_percent),
            current_num_clauses,
            resolved_vars,
            format!("{}ms", inprocessor_time_millis),
        );
        println!(
            "c │ \
            {:<TIME$} │ \
            {:>CONFLICTS_TOTAL$} │ \
            {:>RESTARTS_TOTAL$} │ \
            {:>ASSIGNMENTS_TOTAL$} {:>ASSIGNED_VARS_NUM$} {:<ASSIGNED_VARS_PERC$} │ \
            {:>CLAUSES_CUR$} │ \
            {:>INPROCESSOR_RESOLVED$} {:>INPROCESSOR_TIME$} │",
            "sec.".truecolor(100, 100, 100),
            Self::print_delta(
                self.last_num_conflicts as i32,
                state_statistics.num_conflicts as i32,
                false,
                "",
                "",
            ),
            Self::print_delta(
                self.last_num_restarts as i32,
                state_statistics.num_restarts as i32,
                false,
                "",
                "",
            ),
            Self::print_delta(
                self.last_num_total_assignments as i32,
                state_statistics.num_assignments as i32,
                false,
                "",
                "",
            ),
            Self::print_delta(
                self.last_num_cur_assignments as i32,
                current_num_assignments as i32,
                true,
                "",
                "",
            ),
            Self::print_delta(
                self.last_assigned_vars_percent as i32,
                assigned_vars_percent as i32,
                true,
                "(",
                "%)",
            ),
            Self::print_delta(
                self.last_num_clauses as i32,
                current_num_clauses as i32,
                true,
                "",
                "",
            ),
            Self::print_delta(
                self.last_inprocessor_resolved as i32,
                resolved_vars as i32,
                false,
                "",
                "",
            ),
            Self::print_delta(
                self.last_inprocessor_total_time as i32,
                inprocessor_time_millis as i32,
                false,
                "",
                "ms",
            )
        );

        self.last_num_conflicts = state_statistics.num_conflicts;
        self.last_num_restarts = state_statistics.num_restarts;
        self.last_num_total_assignments = state_statistics.num_assignments;
        self.last_num_cur_assignments = current_num_assignments;
        self.last_assigned_vars_percent = assigned_vars_percent;
        self.last_num_clauses = current_num_clauses;
        self.last_inprocessor_resolved = resolved_vars;
        self.last_inprocessor_total_time = inprocessor_time_millis;
    }

    pub fn close_table(&self) {
        println!(
            "c └─\
            {:─<TIME$}─┴─\
            {:─<CONFLICTS_TOTAL$}─┴─\
            {:─<RESTARTS_TOTAL$}─┴─\
            {:─<ASSIGNMENTS_TOTAL$}─{:─<ASSIGNED_VARS_NUM$}─{:─<ASSIGNED_VARS_PERC$}─┴─\
            {:─<CLAUSES_CUR$}─┴─\
            {:─<INPROCESSOR_RESOLVED$}─{:─<INPROCESSOR_TIME$}─┘",
            "", "", "", "", "", "", "", "", ""
        );
    }

    fn print_delta(
        old_value: i32,
        new_value: i32,
        use_colors: bool,
        prepend: &str,
        append: &str,
    ) -> ColoredString {
        let mut output = prepend.to_string();

        let delta = new_value - old_value;

        if delta >= 0 {
            output.push('+');
        }

        output.push_str(&delta.to_string());
        output.push_str(append);

        if delta >= 0 && use_colors {
            output.truecolor(0, 150, 0)
        } else if delta < 0 && use_colors {
            output.truecolor(150, 0, 0)
        } else {
            output.yellow()
        }
    }
}
