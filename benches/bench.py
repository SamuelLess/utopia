import datetime
import math
import os
import subprocess
import sys
from time import time
from typing import List

import pandas as pd
import tqdm
from matplotlib import pyplot as plt
from plotnine import *

plt.rcParams['text.usetex'] = True
plt.rcParams["font.family"] = "serif"

TIMEOUT = 1
FILENAME = f"results-{datetime.datetime.now().strftime('%Y-%m-%d-%H-%M-%S')}.csv"

# This a python script to create cactus plots to compare various versions
# of the solver. It's not implemented in Rust to be able to spawn the solver
# process and be able to accurately kill it after the timeout expired.
# Also, it can run other solvers this way, too.

ARCANE_BIN_PATH = "../../arcane/target/release/arcane"
UTOPIA_BIN_PATH = "../target/release/utopia"

SOLVERS = {
    'arcane-baseline': [ARCANE_BIN_PATH, "--heuristic", "naive", "--disable-pure-literal-elimination"],
    'arcane-ple': [ARCANE_BIN_PATH, "--heuristic", "naive"],
    'arcane-dlis': [ARCANE_BIN_PATH, "--heuristic", "dlis"],
    'arcane-dlcs': [ARCANE_BIN_PATH, "--heuristic", "dlcs"],
    'arcane-moms': [ARCANE_BIN_PATH, "--heuristic", "moms"],
    'arcane-jw': [ARCANE_BIN_PATH, "--heuristic", "jeroslaw-wang"],
    'arcane-decay': [ARCANE_BIN_PATH, "--heuristic", "decay"],
    'arcane-decay-no-prep': [ARCANE_BIN_PATH, "--heuristic", "decay", "--disable-preprocessor"],
    'arcane-dyn-clause-len': [ARCANE_BIN_PATH, "--heuristic", "dyn-clause-len"],
    'arcane-clause-len': [ARCANE_BIN_PATH, "--heuristic", "clause-len"],
    'arcane-static-occ': [ARCANE_BIN_PATH, "--heuristic", "static-occ"],
    'utopia': [UTOPIA_BIN_PATH],
    'utopia-no-inprocessing': [UTOPIA_BIN_PATH, "--no-inprocessing"],
    'utopia-proof': [UTOPIA_BIN_PATH, "--proof", "out.proof"],
    'utopia-fixed-restarts': [UTOPIA_BIN_PATH, "--restart-policy", "fixed-interval"],
    'utopia-geometric-restarts': [UTOPIA_BIN_PATH, "--restart-policy", "geometric"],
    'utopia-luby-restarts': [UTOPIA_BIN_PATH, "--restart-policy", "luby"],
    'utopia-no-restarts': [UTOPIA_BIN_PATH, "--restart-policy", "no-restarts"],
    'cadical': ['cadical'],
    'minisat': ['minisat'],
    'z3': ['z3'],
}

BENCHMARK_SETS = {
    'lecture': "../testfiles/lecture_testfiles",
    '2006': "../testfiles/competitions/2006"
}


def find_files(path, filters=[]):
    """find all cnf files in the lecture_testfiles directory"""
    cnf_files = []
    for root, dirs, files in os.walk(path):
        for file in files:
            if not (file.endswith(".cnf") or file.endswith(".cnf.gz")):
                continue
            full_path = os.path.join(root, file)
            skip = False
            for f in filters:
                if f in full_path:
                    skip = True
            if skip:
                continue
            cnf_files.append(full_path)
        # random.shuffle(cnf_files)
    return cnf_files


# define a function to run our solver
def solve_with_binary(binary: List[str], cnf_file):
    # call the solver with the file as a parameter. Return the time it took to solve the file
    # abort after TIMEOUT second

    start_time = time()
    assignments = 0
    try:
        process = subprocess.run([*binary, cnf_file], stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=TIMEOUT)
        true_result = None
        if "lecture_testfiles" in cnf_file:
            true_result = "unsat" if "unsat" in cnf_file else "sat"
        if "2006" in cnf_file:
            if "sat" in cnf_file:
                true_result = "sat"
            elif "uns" in cnf_file:
                true_result = "unsat"

        contains_unsat = "UNSATISFIABLE" in process.stdout.decode('utf-8') or "unsat" in process.stdout.decode('utf-8')
        contains_sat = ("SATISFIABLE" in process.stdout.decode('utf-8') or
                        ("sat" in process.stdout.decode('utf-8') and not contains_unsat))

        solver_result = "unsat" if contains_unsat else "sat" if contains_sat and not contains_unsat else "UNKNOWN"

        if true_result is not None:
            if true_result != solver_result:
                tqdm.tqdm.write(f'Wrong result for {cnf_file}: {solver_result} instead of {true_result}')
                return f"WRONG RESULT", 0

        # get assignment count from output
        if contains_sat:
            for line in process.stdout.decode('utf-8').splitlines():
                if 'Assignments' in line:
                    assignments = int(line.split()[-1])
                    break

    except subprocess.TimeoutExpired:
        tqdm.tqdm.write(f'Timeout while solving {cnf_file}')
        return math.inf, math.inf
    end_time = time()
    return end_time - start_time, assignments


def solve(solver, file):
    if solver in SOLVERS:
        return solve_with_binary(SOLVERS[solver], file)
    else:
        raise ValueError(f"Unknown solver: {solver}, available solvers: {SOLVERS.keys()}")


def read_or_create_checkpoint() -> List[dict]:
    if os.path.exists(FILENAME):
        return pd.read_csv(FILENAME).to_dict('records')
    else:
        return pd.DataFrame(columns=['solver', 'time', 'file']).to_dict('records')


def create_plot(data, show=True, assignments=False, solvers=[]):
    # calculate the cumulative time for each solver and the rank of each file per solver
    key = 'assignments' if assignments else 'time'

    df = pd.DataFrame(data)

    if len(df) == 0:
        return
    # only keep rows that contain lecture in the filename in the dataframe
    # df.drop(df[~df['file'].str.contains("lecture")].index, inplace=True)
    # df.drop(df[df['file'].str.contains("/test/")].index, inplace=True)

    # remove all inf values
    df = df[df[key] != math.inf]
    df = df[df[key] != 'inf']
    print(df[df[key] == "WRONG RESULT"])
    df = df[df[key] != 'WRONG RESULT']
    df[key] = pd.to_numeric(df[key])
    df = df.sort_values(by=['solver', key])
    # df = df.sort_values(by=['solver', 'file'])
    df[f'cumulative_{key}'] = df.groupby('solver')[key].cumsum()
    # df['rank'] = df.groupby('solver')[f'cumulative_{key}'].rank(method='dense')
    df['rank'] = df.groupby('solver')[f'{key}'].rank(method='average')

    # only take solvers that are in the solvers list
    df = df[df['solver'].isin(solvers)]

    # rename 'arcane-dyn-clause-len' to 'dyn-clause-len' for better plotting
    # remove 'arcane-' from the solver names
    df['solver'] = df['solver'].str.replace('arcane-', '')

    # plot the results
    plot = (ggplot(df, aes(x='rank', y=f'{key}', color='solver', shape='solver')) +
            geom_point(size=0.75) +
            geom_line() +
            # lims(x=(0,171)) +
            # scale_y_log10(limits=(0.1, 500)) +
            scale_y_log10() +
            labs(x='Solved instances', y='CPU time (s)', color="Solver", shape="Solver", title='Utopia runtime') +
            theme_bw() +
            theme(legend_position='bottom')
            )
    # change colormap
    plot = plot + scale_color_manual(
        values=["#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#9467bd", "#8c564b", "#e377c2", "#7f7f7f", "#bcbd22"])

    plot.save(f'{FILENAME}.svg', verbose=False)
    if show:
        plot.draw(show=show)


last_intermediate_plot_creation = 0


def create_plot_occasionally(data, solvers):
    global last_intermediate_plot_creation
    if time() - last_intermediate_plot_creation > 30:
        last_intermediate_plot_creation = time()
        if len(data) > 2:
            create_plot(data, show=False, solvers=solvers)


def main():
    solvers = []
    benchmarks = []

    arguments = sys.argv[1:]
    # validate solvers
    for arg in arguments:
        if arg in SOLVERS:
            solvers.append(arg)
        elif arg in BENCHMARK_SETS:
            benchmarks.append(arg)
        else:
            print(f"Unknown argument: {arg}")
            return

    if len(benchmarks) == 0:
        benchmarks = ["lecture"]
    if len(solvers) == 0:
        solvers = ["utopia"]

    cnf_files = []
    for benchmark_set in benchmarks:
        print(f"Reading benchmarks from {benchmark_set}")
        cnf_files += find_files(path=BENCHMARK_SETS[benchmark_set])

    data = read_or_create_checkpoint()
    for solver in tqdm.tqdm(solvers):
        for cnf_file in tqdm.tqdm(cnf_files, desc=f'Benchmarking {solver} '):
            # df doesn't contain this solver-file combination yet
            if list(filter(lambda x: x['solver'] == solver and x['file'] == cnf_file, data)):
                continue

            _time, assignments = solve(solver, cnf_file)
            data.append({'solver': solver, 'time': _time, 'file': cnf_file, 'assignments': assignments})

            # save the data
            # sort data by time
            data = sorted(data, key=lambda x: str(x['time']))
            pd.DataFrame(data).to_csv(FILENAME, index=False)

            # create the plot
            create_plot_occasionally(data, solvers)

    create_plot(data, assignments=False, solvers=solvers)

    print(f"Total time: {sum([x['time'] for x in data if x['time'] != math.inf])}")


if __name__ == "__main__":
    main()
