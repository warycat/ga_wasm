#[macro_use]
extern crate lazy_static;

extern crate rand;
use genevo::{operator::prelude::*, prelude::*};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, Window};

mod tsp;
use tsp::{City, Sequence, RandomSequence, TSP, RenderSequence};

pub const WIDTH: f64 = 1000.0;
pub const HEIGHT: f64 = 1000.0;
pub const N_CITIES: usize = 80;
pub const N_POPULATION: usize = 40;
pub const N_INDIVIDUALS_PER_PARENTS: usize = 3;
pub const SELECTION_RATIO: f64 = 0.7;
pub const MUTATION_RATE: f64 = 0.05;
pub const REINSERTION_RATIO: f64 = 0.7;
pub const GENERATION_LIMIT: u64 = 10000;


fn window() -> Window {
    web_sys::window().expect("no global `window` exists")
}

fn document() -> Document {
    window()
        .document()
        .expect("should have a document on window")
}

fn canvas() -> HtmlCanvasElement {
    document()
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap()
}

fn context() -> CanvasRenderingContext2d {
    canvas()
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap()
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

lazy_static! {
    /// This is an example for using doc comment attributes
    static ref STATIC_TSP: TSP = TSP::new(N_CITIES, WIDTH, HEIGHT);
    static ref CITIES: Vec<City> = (0..N_CITIES)
        .map(|id| City::random(id, WIDTH, HEIGHT))
        .collect();
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    // Your code goes here!
    let context = context();
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    let mut individuals: Vec<Sequence> = vec![];
    for _ in 0..N_POPULATION {
        individuals.push(Sequence::random(N_CITIES));
    }
    let initial_population: Population<Sequence> = Population::with_individuals(individuals);
    let mut tsp_sim = simulate(
        genetic_algorithm()
            .with_evaluation(&(*STATIC_TSP))
            .with_selection(RouletteWheelSelector::new(
                SELECTION_RATIO,
                N_CITIES,
            ))
            .with_crossover(OrderOneCrossover::new())
            .with_mutation(SwapOrderMutator::new(MUTATION_RATE))
            .with_reinsertion(ElitistReinserter::new(&(*STATIC_TSP), false, REINSERTION_RATIO))
            .with_initial_population(initial_population)
            .build(),
    )
    .until(GenerationLimit::new(GENERATION_LIMIT))
    .build();
    let mut finished = false;
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        match tsp_sim.step() {
            Ok(SimResult::Intermediate(step)) => {
                context.clear_rect(0.0, 0.0, WIDTH, HEIGHT);
                (*STATIC_TSP).render(&context);
                
                context.set_font("Bold 48px serif");
                context.set_fill_style(&"blue".into());
                context.fill_text(&format!("Iteration: {}", step.iteration), 100.0, 100.0).unwrap();
                let best_solution = step.result.best_solution.solution;
                context.set_fill_style(&"grey".into());
                context.fill_text(&format!("Fitness: {}", best_solution.fitness), 100.0, 200.0).unwrap();
                let dna = best_solution.genome;
                dna.render(&(*STATIC_TSP).cities, &context);
            }
            Ok(SimResult::Final(_, _, _, _)) => {
                finished = true;
            }
            Err(error) => {
                println!("{}", error);
            }
        }
        if !finished {
            request_animation_frame(f.borrow().as_ref().unwrap());
        }else{
            return;
        }
    }) as Box<dyn FnMut()>));
    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}
