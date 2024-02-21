use clap::Parser;
use std::collections::HashMap;
use utopia::cnf::{check_assignment, Clause, VarId};
use utopia::dimacs::{clauses_from_dimacs_file, solution_to_dimacs};
use utopia::solver::heuristic::HeuristicType;
use utopia::solver::statistics::StateStatistics;
use utopia::solver::Solver;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(index = 1)]
    file: String,

    /// Output path for solution
    #[arg(short, long, global = true, help = "Output path for solution")]
    out: Option<String>,

    #[arg(long, default_value = "decay")]
    heuristic: HeuristicType,
}

fn main() {
    let args = Args::parse();

    let cnf = clauses_from_dimacs_file(&args.file).unwrap();

    let mut solver = Solver::new(cnf.clone(), args.heuristic.clone());

    let solution = solver.solve();

    let output = create_output(&args, cnf, &solution, solver.stats());
    println!("{}", output);
}

fn create_output(
    args: &Args,
    cnf: Vec<Clause>,
    solution: &Option<HashMap<VarId, bool>>,
    stats: &StateStatistics,
) -> String {
    let mut output = format!("c {}", BANNER);
    output.push_str(format!("\nFile\n{}\n", args.file).as_str());
    output.push_str(format!("\n{}", stats.to_table()).as_str());
    // verify solution
    if let Some(solution) = solution.clone() {
        if check_assignment(cnf.clone(), solution) {
            output.push_str("Solution has been verified and is correct");
        } else {
            output.push_str("WRONG SOLUTION");
        }
    }
    output = output.replace('\n', "\nc ");
    output.push_str(format!("\n{}", solution_to_dimacs(solution.clone())).as_str());
    output
}

const BANNER: &str = r#"
          _                     
    _   _| |_ ___  _ __  _  __ _ 
   | | | | __/ _ \| '_ \| |/ _` |
   | |_| | || (_) | |_) | | (_| |
    \__,_|\__\___/| .__/|_|\__,_|
                  |_|            
"#;
