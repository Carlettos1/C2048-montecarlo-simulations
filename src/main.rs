use c2048::*;
use csta::prelude::*;

fn main() {
    let controller = sim::Controller::new((0..400).map(|i| i as f64 / 100.0).collect());
    controller.launch();
}

pub fn sim() {
    let n_tries = 1_000_000;
    let temp = 10;

    let mc = MonteCarlo::default();
    let mut scores = Vec::new();
    let mut maximums = Vec::new();
    let mut steps = Vec::new();

    let mut move_mc = MonteCarlo::default().into_iter();
    let mut temp_mc = MonteCarlo::default().into_iter();

    mc.sample_iter(n_tries).for_each(|mut game: C2048| {
        let mut step = 0;

        loop {
            step += 1;
            let m: Move = move_mc.next().unwrap();
            let posible = game.clone_move(m);
            if !posible.has_moved {
                continue;
            }
            let rn: f64 = temp_mc.next().unwrap();

            if posible.energy() < game.energy()
                || rn < (((-(posible.energy() - game.energy())) / temp) as f64).exp()
            {
                game = posible;
                game.spawn_tile(0.1);
                game.reset();
            }

            if game.is_lose() {
                scores.push(game.score());
                maximums.push(game.highest().exp);
                steps.push(step);
                break;
            }
        }
    });

    println!(
        "scores avg: {}",
        scores.iter().map(|n| *n as f64).sum::<f64>() / scores.len() as f64
    );
    println!(
        "maximum avg: 2^{}",
        maximums.iter().map(|m| *m as f64).sum::<f64>() / maximums.len() as f64
    );
    println!(
        "step avg: {}",
        steps.iter().sum::<usize>() as f64 / steps.len() as f64
    );
}
