use std::{collections::HashMap, sync::mpsc, thread, time::Instant};

use file_log::log;

use super::*;

use crate::C2048;

#[derive(Debug)]
pub struct SimResult {
    energies: Vec<f64>,
    scores: [usize; 14],
    temp: f64,
}

#[derive(Debug)]
pub struct Sim {
    temp: f64,
    id: usize,
    n_iter: usize,
    result: SimResult,
}

impl SimResult {
    fn with_temperature(temp: f64) -> Self {
        Self {
            scores: [0; 14],
            temp,
            energies: Vec::new(),
        }
    }

    pub fn merge(&mut self, other: &mut SimResult) {
        for (other_score, self_score) in other.scores.iter().zip(self.scores.iter_mut()) {
            *self_score += other_score;
        }
        self.energies.append(&mut other.energies);
    }

    fn add(&mut self, (score, energy): (usize, f64)) {
        self.add_score(score);
        self.add_energy(energy);
    }

    fn add_score(&mut self, score: usize) {
        self.scores[score] += 1;
    }

    fn add_energy(&mut self, energy: f64) {
        self.energies.push(energy);
    }
}

impl Sim {
    pub fn new(temp: f64, n_iter: usize, id: usize) -> Self {
        Self {
            temp,
            n_iter,
            id,
            result: SimResult::with_temperature(temp),
        }
    }

    pub fn run(&mut self) {
        let mc = MonteCarlo::default();
        if self.temp == 0.0 {
            mc.sample_iter(self.n_iter)
                .for_each(|game| self.result.add(inner_sim_at_0(game)))
        } else {
            mc.sample_iter(self.n_iter)
                .for_each(|game| self.result.add(inner_sim(self.temp, game)));
        }
        println!("({: >7})Batch {: >3} ended", self.temp, self.id)
    }
}

fn inner_sim(temp: f64, mut game: C2048) -> (usize, f64) {
    let mut energies = Vec::new();
    let mut move_mc = MonteCarlo::default().into_iter();
    let mut temp_mc = MonteCarlo::default().into_iter();
    let mut counter = 0;

    loop {
        if counter == 4 {
            perfect_play!(game, energies);
            counter = 0;
            continue;
        }

        let m: Move = move_mc.next().unwrap();
        let posible = game.clone_move(m);
        if !posible.has_moved {
            counter += 1;
            continue;
        }
        let rn: f64 = temp_mc.next().unwrap();

        if posible.energy() < game.energy()
            || rn < ((-(posible.energy() - game.energy())) as f64 / temp).exp()
        {
            energies.push(game.energy());
            game = posible;
            game.spawn_tile(0.1);
            game.reset();
            counter = 0;
        }

        if game.is_lose() {
            return (
                game.highest().exp as usize,
                energies.iter().sum::<isize>() as f64 / energies.len() as f64,
            );
        }
        counter += 1;
    }
}

fn inner_sim_at_0(mut game: C2048) -> (usize, f64) {
    let mut energies = Vec::new();
    loop {
        perfect_play!(game, energies);
    }
}

#[macro_export]
macro_rules! perfect_play {
    ($game:tt, $energies:tt) => {
        let up = $game.clone_move(Move::Up);
        let right = $game.clone_move(Move::Right);
        let down = $game.clone_move(Move::Down);
        let left = $game.clone_move(Move::Left);
        let moves = vec![up, right, down, left];
        let min = moves.into_iter().filter(|g| g.has_moved).min();
        if let Some(min) = min {
            $game = min;
            $energies.push($game.energy());
            $game.spawn_tile(0.1);
            $game.reset();
        } else {
            return (
                $game.highest().exp as usize,
                $energies.iter().sum::<isize>() as f64 / $energies.len() as f64,
            );
        }
    };
}

pub struct Controller {
    pub n_threads: usize,
    pub n_iter: usize,
    pub batches: usize,
    pub temperatures: Vec<f64>,
}

impl Controller {
    pub fn new(temperatures: Vec<f64>) -> Self {
        Self {
            n_threads: thread::available_parallelism().unwrap().into(),
            n_iter: 100,
            batches: 100,
            temperatures,
        }
    }

    pub fn launch(&self) {
        assert!(
            self.n_threads < self.temperatures.len() * self.batches,
            "THREADS must be lower than TEMPERATURES * BATCHES"
        );

        let now = Instant::now();
        let mut map = HashMap::new();
        for temp in self.temperatures.iter() {
            map.insert(
                (temp * 10000.0) as isize,
                SimResult::with_temperature(*temp),
            );
        }

        let mut temp_cicle = self
            .temperatures
            .clone()
            .into_iter()
            .cycle()
            .enumerate()
            .take(self.temperatures.len() * self.batches);

        let (sender, receiver) = mpsc::channel();
        let mut sender = Some(sender);

        for _ in 0..self.n_threads {
            let sender = sender.clone().unwrap();
            let (id, temp) = temp_cicle.next().unwrap();
            let mut sim = Sim::new(temp, self.n_iter, id / self.temperatures.len());
            thread::spawn(move || {
                sim.run();
                sender.send(sim.result)
            });
        }

        for mut result in receiver {
            for i in 0..14 {
                map.get_mut(&((result.temp * 10000.0) as isize))
                    .unwrap_or_else(|| panic!("{i}-{} not exist in map", result.temp))
                    .merge(&mut result);
            }

            let temp = temp_cicle.next();

            if let Some((id, temp)) = temp {
                let sender = sender.clone().unwrap();
                let mut sim = Sim::new(temp, self.n_iter, id / self.temperatures.len());
                thread::spawn(move || {
                    sim.run();
                    sender.send(sim.result)
                });
            } else if sender.is_some() {
                sender = None;
            }
        }

        let mut vec: Vec<(isize, SimResult)> = map.into_iter().collect();
        vec.sort_by(|(a1, _), (b1, _)| a1.cmp(b1));
        for v in vec {
            let energy_avg = v.1.energies.iter().sum::<f64>() / v.1.energies.len() as f64;
            println!("score ({}, {:?})", v.0, v.1.scores);
            println!("energy avg of avg {}, {}", v.0, energy_avg);
            log!("energy_avg", "{}", energy_avg);
            log!("victories", "{}", v.1.scores[11]);
        }

        println!("Everything ended! :D {:?}", now.elapsed());
    }
}
