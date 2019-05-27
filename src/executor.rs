use std::rc::Rc;
use crate::maps::Map;
use crate::Algorithm;
use crate::Scene;
use crate::AlgoStatus;
use crate::Renderer;
use crate::Coord;
use crate::SceneParams;
use crate::SceneParams::Empty;
use crate::max;
use ggez::{Context, GameError, timer};
use ggez::event::KeyCode;
use std::cell::RefCell;
use crate::{point, CELL_SIZE, GRID_VERT_COUNT, GRID_HORZ_COUNT, GRID_START, SCREEN_WIDTH, SCREEN_HEIGHT};
use ggez::graphics::{Text, TextFragment, Color, Scale, MeshBuilder, DrawMode, Rect};

pub struct Executor {
    map: Rc<Map>,
    algo: Rc<RefCell<Algorithm>>,
    auto_advance: bool,
    advance: bool,
    update_speed: f64,
    last_update: f64,
    ticks: usize,
    algo_name: String,
}

impl Executor {
    pub fn new(map: Rc<Map>, algo: Rc<RefCell<Algorithm>>, algo_name: String) -> Executor {
        Executor {
            map,
            algo,
            auto_advance: true,
            advance: false,
            update_speed: 0.2,
            last_update: 0.,
            ticks: 0,
            algo_name,
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
        let step_text;
        match self.algo.borrow().get_data() {
            AlgoStatus::InProgress(_) => step_text = format!("Tick {}", self.ticks),
            AlgoStatus::Found(_) => step_text = format!("Found in {} ticks", self.ticks),
            AlgoStatus::NoPath => step_text = format!("Failed after {} ticks", self.ticks)
        }
        let display = format!("Map: {} - Algo: {}  |  {}  |  {} ", self.map.idx, self.algo_name, advancing_text, step_text);
        renderer.draw_text(ctx, display, point(8., 4.));
    }
}

fn draw_start_ends(start: Coord, targets: Vec<Coord>, ctx: &mut Context, renderer: &mut Renderer) -> Result<(), GameError> {
    let indicator_mesh = renderer.make_square_mesh(ctx, CELL_SIZE, false, 2.)?;
    renderer.draw_coloured_mesh(ctx, indicator_mesh.as_ref(), point(GRID_START.0 + (start.x as f32 * CELL_SIZE), GRID_START.1 + (start.y as f32 * CELL_SIZE)), (0.5, 1., 0.5, 1.).into());
    for target in targets {
        renderer.draw_coloured_mesh(ctx, indicator_mesh.as_ref(), point(GRID_START.0 + (target.x as f32 * CELL_SIZE), GRID_START.1 + (target.y as f32 * CELL_SIZE)), (0.5, 0.5, 1., 1.).into());
    }
    Ok(())
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
        let grid_mesh = renderer.make_grid_mesh(ctx, CELL_SIZE, GRID_HORZ_COUNT, GRID_VERT_COUNT, 255)?;
        renderer.draw_mesh(ctx, grid_mesh.as_ref(), point(GRID_START.0, GRID_START.1));

        let square_mesh = renderer.make_square_mesh(ctx, CELL_SIZE, true, 2.)?;
        for map_x in 0..GRID_HORZ_COUNT {
            for map_y in 0..GRID_VERT_COUNT {
//                if self.map.cost[map_x][map_y] < 0 {
//                    renderer.draw_mesh(ctx, square_mesh.as_ref(), point(GRID_START.0 + (map_x as f32 * CELL_SIZE), GRID_START.1 + (map_y as f32 * CELL_SIZE)));
//                }
                let cost = self.map.cost[map_x][map_y];
                if cost < 0 {
                    renderer.draw_mesh(ctx, square_mesh.as_ref(), point(GRID_START.0 + (map_x as f32 * CELL_SIZE), GRID_START.1 + (map_y as f32 * CELL_SIZE)));
                } else if cost > 0 {
                    let cost_perc = cost as f32 / 10.;
                    let color = (1.,1.,1., cost_perc);
                    renderer.draw_coloured_mesh(ctx, square_mesh.as_ref(), point(GRID_START.0 + (map_x as f32 * CELL_SIZE), GRID_START.1 + (map_y as f32 * CELL_SIZE)), color.into());
                }
            }
        }

        match self.algo.borrow().get_data() {
            AlgoStatus::InProgress((open_nodes, closed_nodes)) => {
                let open_color = (0.5, 0.5, 0.7, 0.8).into();
                let closed_color = (0.3, 0.3, 0.5, 0.8).into();

                for open in open_nodes {
                    renderer.draw_coloured_mesh(ctx, square_mesh.as_ref(), point(GRID_START.0 + (open.x as f32 * CELL_SIZE), GRID_START.1 + (open.y as f32 * CELL_SIZE)), open_color);
                }
                for closed in closed_nodes {
                    renderer.draw_coloured_mesh(ctx, square_mesh.as_ref(), point(GRID_START.0 + (closed.x as f32 * CELL_SIZE), GRID_START.1 + (closed.y as f32 * CELL_SIZE)), closed_color);
                }
                draw_start_ends(self.map.start, self.map.targets.clone(), ctx, renderer)?;
            }
            AlgoStatus::Found(path) => {
                let path_color = (0.5, 1.0, 0.5, 0.9).into();
                for step in path {
                    renderer.draw_coloured_mesh(ctx, square_mesh.as_ref(), point(GRID_START.0 + (step.x as f32 * CELL_SIZE), GRID_START.1 + (step.y as f32 * CELL_SIZE)), path_color);
                }
                draw_start_ends(self.map.start, self.map.targets.clone(), ctx, renderer)?;
            }
            AlgoStatus::NoPath => {
                let text = Text::new(TextFragment {
                    text: String::from("No path found"),
                    color: Some(Color::new(1., 0., 0., 1.)),
                    scale: Some(Scale::uniform(60.)),
                    ..TextFragment::default()
                });
                let mesh = MeshBuilder::new().rectangle(DrawMode::fill(), Rect::new(0., 0., SCREEN_WIDTH, SCREEN_HEIGHT * 0.12), (0, 0, 0).into()).build(ctx)?;
                renderer.draw_mesh(ctx, &mesh, point(0., SCREEN_HEIGHT * 0.44));
                renderer.draw_mesh(ctx, &text, point(SCREEN_WIDTH * 0.5 - 150., SCREEN_HEIGHT * 0.47));
            }
        }

        self.draw_info_text(ctx, renderer);

        Ok(())
    }

    fn on_button_press(&mut self, keycode: KeyCode) {
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

    fn is_complete(&self) -> bool {
        false
    }

    fn get_next_stage_params(&self) -> SceneParams {
        Empty
    }
}