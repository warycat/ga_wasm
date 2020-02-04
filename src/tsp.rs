use genevo::{prelude::*, random::Rng};
use rand::rngs::OsRng;
use rand::seq::SliceRandom;
use std::f64::consts::PI;
use web_sys::CanvasRenderingContext2d;

#[derive(Debug, Clone)]
pub struct City {
    id: usize,
    x: f64,
    y: f64,
}

impl City {
    pub fn random(id: usize, max_x: f64, max_y: f64) -> Self {
        let mut rng = OsRng;
        let mut x: f64 = rng.gen();
        let mut y: f64 = rng.gen();
        x *= max_x;
        y *= max_y;
        City { id, x, y }
    }
    fn render(&self, context: &CanvasRenderingContext2d) {
        context.set_fill_style(&"red".into());
        context.begin_path();
        context.arc(self.x, self.y, 4.0, 0.0, 2.0 * PI).unwrap();
        context.fill();
    }

    fn distance_squared(&self, other: &City) -> f64 {
        (self.x - other.x) * (self.x - other.x) + (self.y - other.y) * (self.y - other.y)
    }
}

pub type Sequence = Vec<usize>;

pub trait RenderSequence {
    fn render(&self, cities: &Vec<City>, context: &CanvasRenderingContext2d);
}

impl RenderSequence for Sequence {
    fn render(&self, cities: &Vec<City>, context: &CanvasRenderingContext2d) {
        let n = cities.len();
        context.begin_path();
        context.move_to(cities[self[0]].x, cities[self[0]].y);
        for i in 1..n {
            context.line_to(cities[self[i]].x, cities[self[i]].y);
        }
        context.stroke();
    }
}

pub trait RandomSequence {
    fn random(n: usize) -> Vec<usize>;
}

impl RandomSequence for Sequence {
    fn random(n: usize) -> Vec<usize> {
        let mut dna: Vec<usize> = (0..n).collect();
        dna.shuffle(&mut OsRng);
        dna
    }
}

#[derive(Debug, Clone)]
pub struct TSP {
    pub cities: Vec<City>,
}

impl TSP {
    pub fn new(n: usize, width: f64, height: f64) -> Self {
        let cities: Vec<City> = (0..n)
            .map(|id| City::random(id, width, height))
            .collect();
        TSP { cities }
    }
    pub fn render(&self, context: &CanvasRenderingContext2d) {
        for city in &self.cities{
            city.render(context);
        }
    }
}

impl<'a> FitnessFunction<Sequence, i64> for &'a TSP {
    fn fitness_of(&self, sequences: &Sequence) -> i64 {
        let mut sum: f64 = 0.0;
        for w in sequences.windows(2) {
            sum += self.cities[w[0]].distance_squared(&self.cities[w[1]]);
        }
        (std::i64::MAX as f64 / sum + 0.5).floor() as i64
    }
    fn average(&self, values: &[i64]) -> i64 {
        (values.iter().sum::<i64>() as f64 / values.len() as f64) as i64
    }

    fn highest_possible_fitness(&self) -> i64 {
        std::i64::MAX
    }

    fn lowest_possible_fitness(&self) -> i64 {
        0
    }
}

