use std::env;
use std::fmt;

use itertools::join;
// use rand::{Fill, Rng, SeedableRng};
// use rand_chacha::ChaCha8Rng;

use good_lp::solvers::coin_cbc::coin_cbc;
use good_lp::{
    constraint, default_solver, variable, Constraint, ProblemVariables, Solution, SolverModel,
};

fn pretty<T: fmt::Display, const X: usize, const Y: usize>(arr: &[[T; X]; Y], prec: usize) {
    for inner in arr.iter() {
        println!(
            "{}",
            join(inner.iter().map(|el: &T| format!("{:.prec$}", el)), ", ")
        );
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // let n: usize = 10;
    let n: usize = args[1].parse().expect("Bad argument, integer !> 0");
    let mut set_timeout = false;
    if args.len() > 2 {
	match args[2].parse::<u16>() {
	    Ok(timeout) => if timeout > 0 { set_timeout = true; }
	    Err(error) => println!("Ignoring bad flag, not a bool: {error}")
	};
    }

    /*
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut _arr = [[0u16; n]; n];
    for row in _arr.iter_mut() {
        match row.try_fill(&mut rng) {
            Ok(_) => (),
            Err(error) => panic!("Could not allocate {:?}", error),
        };
        for i in row.iter_mut() {
            *i %= 2;
        }
        // row.map(|&mut i| *i %= 2);
    }

    if n <= 10 {
        pretty(&_arr, 5);
    }
    */

    let mut probvars = ProblemVariables::new();
    let vars = probvars.add_vector(variable().binary(), n * n);
    let mut constraints: Vec<Constraint> = Vec::with_capacity(n + n + 2 * (2 * n - 1));
    for i in 0..n {
        let lhs_col = (0..n)
            .take_while(|j| (i + j * n) < n * n)
            .map(|j| 1 * vars[i + j * n])
            .reduce(|v1, v2| v1 + v2)
            .unwrap();
        constraints.push(constraint!(lhs_col == 1));

        let lhs_row = (0..n)
            .take_while(|j| (i * n + j) < n * n)
            .map(|j| 1 * vars[i * n + j])
            .reduce(|v1, v2| v1 + v2)
            .unwrap();
        constraints.push(constraint!(lhs_row == 1));

        let lhs_upper_diag = (0..n)
            .take_while(|j| (i + j * n + j) < n * n)
            .map(|j| 1 * vars[i + j * n + j])
            .reduce(|v1, v2| v1 + v2)
            .unwrap();
        constraints.push(constraint!(lhs_upper_diag <= 1));

        if i > 0 {
            let lhs_lower_diag = (0..n)
                .take_while(|j| (i * n + j * n + j) < n * n)
                .map(|j| 1 * vars[i * n + j * n + j])
                .reduce(|v1, v2| v1 + v2)
                .unwrap();
            constraints.push(constraint!(lhs_lower_diag <= 1));
        }
    }

    let objective = (0..n).map(|i| 1 * vars[i]).reduce(|i, j| i + j).unwrap();

    // let mut solver_problem = probvars.maximise(objective).using(default_solver);

    // let solver = CbcSolver::new().with_max_seconds(0);
    let mut solver_problem = coin_cbc(probvars.maximise(objective));
    if set_timeout {
	solver_problem.set_parameter("seconds", "0");
    }

    // trait: good_lp::SolverModel
    for _ in 0..constraints.len() {
        solver_problem = solver_problem.with(constraints.pop().unwrap());
    }
    match solver_problem.solve() {
        Ok(solution) => {
            if n <= 10 {
                for i in 0..n {
                    // trait: good_lp::Solution
                    println!(
                        "{:?}",
                        vars[i * n..]
                            .into_iter()
                            .take(n)
                            .map(|v| if solution.value(*v) > 0.1 { 1 } else { 0 })
                            .collect::<Vec<u16>>()
                    );
                }
            }
        }
        Err(error) => println!("Exiting prematurely: {error:?}"),
    };
}
