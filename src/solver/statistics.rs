use prettytable::{row, Table};

#[derive(Debug, Clone)]
pub struct StateStatistics {
    pub num_clauses: usize,
    pub num_vars: usize,
    pub num_backtracks: usize,
    pub num_conflicts: usize,
    pub num_decisions: usize,
    pub num_propagations: usize,
    pub num_assignments: usize,
    pub num_unassignments: usize,
    pub num_ple: usize,
    pub start_time: std::time::Instant,
    pub time: std::time::Duration,
}

impl Default for StateStatistics {
    fn default() -> Self {
        StateStatistics {
            num_clauses: 0,
            num_vars: 0,
            num_backtracks: 0,
            num_conflicts: 0,
            num_decisions: 0,
            num_propagations: 0,
            num_assignments: 0,
            num_unassignments: 0,
            num_ple: 0,
            start_time: std::time::Instant::now(),
            time: std::time::Duration::from_secs(123),
        }
    }
}

impl StateStatistics {
    pub fn new(num_clauses: usize, num_vars: usize) -> Self {
        StateStatistics {
            num_clauses,
            num_vars,
            ..Default::default()
        }
    }

    pub fn stop_timing(&mut self) {
        self.time = self.start_time.elapsed();
    }

    pub fn start_timing(&mut self) {
        self.start_time = std::time::Instant::now();
    }

    pub fn to_table(&self) -> Table {
        let mut table = Table::new();
        table.set_format(*prettytable::format::consts::FORMAT_NO_COLSEP);
        table.set_titles(row![b -> "Solver Statistics", "Value"]);
        if self.num_clauses == 0 {
            table.add_row(row!["No Data - Only Preprocessing"]);
            return table;
        }
        table.add_row(row![
            "Size",
            format!("{} clauses, {} vars", self.num_clauses, self.num_vars)
        ]);

        // each row with name -> property
        table.add_row(row!["Assignments", self.num_assignments]);
        table.add_row(row!["Conflicts", self.num_conflicts]);
        table.add_row(row![
            "Correct Decisions",
            if self.num_decisions as i32 - self.num_backtracks as i32 > 0 {
                self.num_decisions - self.num_backtracks
            } else {
                0
            }
        ]);
        table.add_row(row!["Propagations", self.num_propagations]);
        table.add_row(row![
            "Time (approx.)",
            format!("{:.3}s", self.time.as_secs_f32())
        ]);
        table
    }
}
