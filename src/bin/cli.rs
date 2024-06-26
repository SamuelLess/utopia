use clap::{arg, Parser};
use std::collections::HashMap;
use utopia::cnf::{check_assignment, Clause, VarId};
use utopia::dimacs::{clauses_from_dimacs_file, solution_to_dimacs};
use utopia::solver::config::Config;
use utopia::solver::heuristic::HeuristicType;
use utopia::solver::progress::ProgressPrintingInterval;
use utopia::solver::restarts::RestartPolicy;
use utopia::solver::statistics::StateStatistics;
use utopia::solver::Solver;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(index = 1)]
    file: String,

    /// Proof file
    #[arg(short, long, help = "Path to put proof file")]
    proof: Option<String>,

    #[arg(long, default_value = "vsids")]
    heuristic: HeuristicType,

    #[arg(short, long, default_value = "glucose-ema")]
    restart_policy: RestartPolicy,

    #[arg(long, default_value = "false")]
    no_inprocessing: bool,

    #[arg(long, default_value = "medium")]
    progress_printing: ProgressPrintingInterval,
}

fn main() {
    let args = Args::parse();

    println!("{}", BANNER);
    let dimacs = clauses_from_dimacs_file(&args.file).unwrap();

    let mut solver = Solver::new(
        dimacs.clauses.clone(),
        dimacs.num_vars,
        Config::new(
            args.heuristic.clone(),
            args.proof.clone(),
            args.restart_policy,
            !args.no_inprocessing,
            args.progress_printing.clone(),
        ),
    );

    let solution = solver.solve();

    let output = create_output(&args, dimacs.clauses, &solution, solver.stats());
    println!("{}", output);
}

fn create_output(
    args: &Args,
    cnf: Vec<Clause>,
    solution: &Option<HashMap<VarId, bool>>,
    stats: &StateStatistics,
) -> String {
    let mut output = String::new();
    output.push_str(format!("\nFile\n{}\n", args.file).as_str());
    output.push_str(format!("\n{}\n", stats.to_table()).as_str());
    // verify solution
    if let Some(solution) = solution.clone() {
        if check_assignment(&cnf, solution) {
            output.push_str("Solution has been verified and is correct\n");
        } else {
            output.push_str("WRONG SOLUTION\n");
        }
    } else if let Some(out) = args.proof.clone() {
        output.push_str(format!("Proof has been written to:\n {}\n", out).as_str());
    }

    output = output.replace('\n', "\nc ");
    output.push_str(format!("\n{}", solution_to_dimacs(solution.clone())).as_str());
    output
}

const BANNER: &str = r#"c          _                     
c    _   _| |_ ___  _ __  _  __ _ 
c   | | | | __/ _ \| '_ \| |/ _` |
c   | |_| | || (_) | |_) | | (_| |
c    \__,_|\__\___/| .__/|_|\__,_|
c                  |_|            
c"#;
