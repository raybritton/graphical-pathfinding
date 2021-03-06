pub mod astar;
pub mod dijkstra;

use crate::data::Coord;

pub type CostCalc = dyn Fn(Coord) -> i32;

pub enum AlgoStatus {
    InProgress((Vec<Coord>, Vec<Coord>)),
    Found(Vec<Coord>, Vec<Coord>),
    NoPath,
}

pub trait Algorithm {
    fn tick(&mut self);
    fn get_data(&self) -> &AlgoStatus;
}

#[derive(Debug, Clone, Copy)]
pub enum Algo {
    AStar,
    Dijkstra
}

impl Algo {
    pub fn name(&self) -> String {
        return match self {
            Algo::AStar => String::from("A*"),
            Algo::Dijkstra => String::from("Dijkstra")
        };
    }

    pub fn len() -> usize {
        2
    }

    pub fn from_index(idx: usize) -> Algo {
        return match idx {
            0 => Algo::AStar,
            1 => Algo::Dijkstra,
            _ => panic!("Invalid index: {}", idx),
        };
    }

    pub fn supported_heuristics(&self) -> bool {
        match self {
            Algo::AStar => true,
            Algo::Dijkstra => false
        }
    }
}
