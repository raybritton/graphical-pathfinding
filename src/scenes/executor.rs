use std::rc::Rc;
use crate::data::maps::Map;
use crate::algos::Algorithm;
use crate::scenes::{Scene, SceneParams::EndOfProgram};
use crate::algos::AlgoStatus;
use crate::graphics::renderer::Renderer;
use crate::scenes::SceneParams;
use crate::{max, GRID_HORZ_COUNT, GRID_VERT_COUNT};
use ggez::{Context, GameError, timer};
use ggez::event::KeyCode;
use std::cell::RefCell;
use crate::{point, SCREEN_WIDTH, SCREEN_HEIGHT};
use ggez::graphics::{Text, TextFragment, Color, Scale, MeshBuilder, DrawMode, Rect};
use crate::graphics::map_rendering::{draw_map_with_costs_nodes, draw_map_with_costs_path, draw_map_with_costs_start_end};
use std::collections::HashMap;

pub struct Executor {
    map_id: usize,
    map: Rc<Map>,
    algo: Rc<RefCell<Box<dyn Algorithm>>>,
    diagonal_mode: String,
    heuristic_mode: String,
    auto_advance: bool,
    advance: bool,
    update_speed: f64,
    last_update: f64,
    ticks: usize,
    algo_name: String,
    variant: usize
}

impl Executor {
    pub fn new(map: Rc<Map>, algo: Rc<RefCell<Box<dyn Algorithm>>>, algo_name: String, diagonal_mode: String, heuristic_mode: String, variant: usize, _cursor_mem: &HashMap<&str, usize>) -> Executor {
        Executor {
            map_id: 0,
            map,
            algo,
            diagonal_mode,
            heuristic_mode,
            auto_advance: true,
            advance: false,
            update_speed: 0.2,
            last_update: 0.,
            ticks: 0,
            algo_name,
            variant
        }
    }
}

impl Executor {
    fn draw_info_text(&mut self, ctx: &mut Context, renderer: &mut Renderer) {
        let advancing_text;
        if self.auto_advance {
            advancing_text = format!("Automatic at {:.1}s", self.update_speed);
        } else {
            advancing_text = String::from("Manual");
        }
        let step_text= match self.algo.borrow().get_data() {
            AlgoStatus::InProgress(_) => format!("{} | Tick {}", advancing_text, self.ticks),
            AlgoStatus::Found(path, _) => format!("Found: {} ticks, Path: {} tiles", self.ticks, path.len()),
            AlgoStatus::NoPath => format!("Failed after {} ticks", self.ticks)
        };
        let display = format!("Map: {}  Algo: {}  Diag: {}  Heur: {}  |  {}", self.map_id, self.algo_name, self.diagonal_mode, self.heuristic_mode, step_text);
        renderer.draw_white_text(ctx, display, point(8., 4.), renderer.calc_height(0.04), false);
    }
}

impl Scene for Executor {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        if !self.auto_advance && !self.advance {
            return Ok(());
        }
        self.advance = false;
        let time = timer::duration_to_f64(timer::time_since_start(ctx));
        if self.advance || (self.last_update + self.update_speed) < time {
            self.last_update = time;

            self.algo.borrow_mut().tick();
            match self.algo.borrow().get_data() {
                AlgoStatus::InProgress(_) => self.ticks += 1,
                _ => {}
            }
        }
        Ok(())
    }

    fn render(&mut self, ctx: &mut Context, renderer: &mut Renderer) -> Result<(), GameError> {
        let cell_size = renderer.calc_width(0.03);
        let grid_width = cell_size * (GRID_HORZ_COUNT as f32);
        let grid_height = cell_size * (GRID_VERT_COUNT as f32);
        let x = renderer.calc_width(0.5) - (grid_width * 0.5);
        let y = renderer.calc_height(0.5) - (grid_height * 0.5) + (cell_size * 0.5);
        let grid_start = (x, y);
        match self.algo.borrow().get_data() {
            AlgoStatus::InProgress((open_nodes, closed_nodes)) => {
                draw_map_with_costs_nodes(ctx, renderer, grid_start, cell_size, self.map.clone().as_ref(), open_nodes, closed_nodes, self.variant)?;
            }
            AlgoStatus::Found(path, closed_nodes) => {
                draw_map_with_costs_path(ctx, renderer, grid_start, cell_size, self.map.clone().as_ref(), &path, closed_nodes, self.variant)?;
            }
            AlgoStatus::NoPath => {
                let text = Text::new(TextFragment {
                    text: String::from("No path found"),
                    color: Some(Color::new(1., 0., 0., 1.)),
                    scale: Some(Scale::uniform(60.)),
                    ..TextFragment::default()
                });
                let mesh = MeshBuilder::new().rectangle(DrawMode::fill(), Rect::new(0., 0., SCREEN_WIDTH, SCREEN_HEIGHT * 0.12), (0, 0, 0).into()).build(ctx)?;

                draw_map_with_costs_start_end(ctx, renderer, grid_start, cell_size, self.map.clone().as_ref(), self.variant)?;
                renderer.draw_mesh(ctx, &mesh, point(0., SCREEN_HEIGHT * 0.44));
                renderer.draw_mesh(ctx, &text, point(SCREEN_WIDTH * 0.5 - 150., SCREEN_HEIGHT * 0.47));
            }
        }

        self.draw_info_text(ctx, renderer);

        Ok(())
    }

    fn on_button_down(&mut self, keycode: KeyCode) {
        match keycode {
            KeyCode::P => self.auto_advance = !self.auto_advance,
            KeyCode::Space => {
                self.auto_advance = false;
                self.advance = true;
            }
            KeyCode::LBracket => {
                self.update_speed = max(0., self.update_speed - 0.05);
            }
            KeyCode::RBracket => {
                self.update_speed = max(0., self.update_speed + 0.05);
            }
            _ => {}
        }
    }

    fn on_button_up(&mut self, _keycode: KeyCode) {}

    fn is_complete(&self) -> bool {
        false
    }

    fn get_next_stage_params(&self, _cursor_mem: &mut HashMap<&str, usize>) -> SceneParams {
        EndOfProgram
    }
}