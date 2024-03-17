use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use crate::cnf::{Clause, Literal, VarId};
use itertools::Itertools;

pub struct DimacsFile {
    pub num_vars: usize,
    pub clauses: Vec<Clause>,
}
pub fn clauses_from_dimacs_file(path: &str) -> Result<DimacsFile, String> {
    if !Path::new(path).exists() {
        return Err(format!("File {} not found", path));
    }

    clauses_from_dimacs(if path.ends_with(".gz") {
        let file = std::fs::File::open(path).map_err(|err| err.to_string())?;
        let mut decoder = GzDecoder::new(file);
        let mut result_string = String::new();
        decoder
            .read_to_string(&mut result_string)
            .map_err(|e| e.to_string())?;
        result_string
    } else {
        std::fs::read_to_string(path).map_err(|e| e.to_string())?
    })
}

pub fn clauses_from_dimacs(input: String) -> Result<DimacsFile, String> {
    let mut file_content: Vec<String> = input
        .lines()
        .map(String::from)
        .filter(|line| !line.starts_with('c'))
        .filter(|line| !line.starts_with('%'))
        .filter(|line| !line.is_empty())
        .collect();

    if file_content.last() == Some(&"0".to_string()) {
        file_content.pop();
    }

    // parse header
    let header = file_content
        .first()
        .ok_or("File was empty")?
        .split_whitespace()
        .collect::<Vec<&str>>();

    if header.len() != 4 || header[0] != "p" || header[1] != "cnf" {
        return Err("Invalid DIMACS header".to_string());
    }
    let num_vars = header[2].parse::<usize>().map_err(|err| err.to_string())?;
    let num_clauses = header[3].parse::<usize>().map_err(|err| err.to_string())?;

    let mut clauses: Vec<Vec<Literal>> = file_content
        .iter()
        .filter(|line| !line.starts_with('p')) // filter header
        .join(" ")
        .split_whitespace()
        .map(|lit| lit.parse::<Literal>())
        .collect::<Result<Vec<_>, _>>()
        .map(|lits| {
            lits.split(|lit| (*lit).id() == 0)
                .map(|clause| clause.to_vec())
                .collect_vec()
        })
        .unwrap_or(Vec::new());

    if !clauses.is_empty() && !clauses.last().unwrap().is_empty() {
        return Err("Last clause must end with 0".to_string());
    }
    clauses.pop();

    if clauses.len() != num_clauses {
        return Err(format!(
            "Expected {} clauses, got {}",
            num_clauses,
            clauses.len()
        ));
    }

    // Normalize CNF: sort clauses and remove duplicate literals
    for clause in clauses.iter_mut() {
        clause.sort();
        clause.dedup();
    }

    let var_count_in_clauses = clauses
        .iter()
        .map(|clause| clause.iter().map(|lit| lit.id()).max().unwrap_or(0))
        .max()
        .unwrap_or(0);

    if var_count_in_clauses != num_vars {
        return Err(format!(
            "Expected {} variables, got {}",
            num_vars, var_count_in_clauses
        ));
    }

    let clauses = clauses
        .iter()
        .map(|clause| Clause::from(clause.clone()))
        .collect_vec();

    Ok(DimacsFile { clauses, num_vars })
}

pub fn solution_to_dimacs(solution: Option<HashMap<VarId, bool>>) -> String {
    let mut dimacs = String::new();
    if solution.is_none() {
        return String::from("s UNSATISFIABLE");
    } else {
        dimacs.push_str("s SATISFIABLE\n")
    }
    let assignment = solution.unwrap();
    dimacs.push_str("v ");
    let sorted_vars = assignment
        .iter()
        .sorted_by_key(|(var_id, _)| *var_id)
        .map(|(id, value)| (id, *value))
        .collect_vec();
    for (var_id, value) in sorted_vars.iter() {
        dimacs.push_str(format!("{}{} ", if *value { "" } else { "-" }, var_id).as_str());
        dimacs.push(' ');
    }
    dimacs
}
