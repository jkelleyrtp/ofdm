mod support;
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
    widgets::{
        canvas::{Canvas, Map, MapResolution, Rectangle},
        Block, Borders,
    },
    Terminal,
};

use crossterm::event::{KeyCode, KeyEvent};
use tui::backend::Backend;
use tui::Frame;

use tui_template::tuiapp::TuiApp;

fn main() {
    let mut app = StreamingApp::new();
    app.launch(100).unwrap();
}

pub struct StreamingApp {
    x: f64,
    y: f64,
    ball: Rectangle,
    playground: Rect,
    vx: f64,
    vy: f64,
    dir_x: bool,
    dir_y: bool,
    should_quit: bool,
}

impl StreamingApp {
    fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            ball: Rectangle {
                x: 10.0,
                y: 30.0,
                width: 10.0,
                height: 10.0,
                color: Color::Yellow,
            },
            playground: Rect::new(10, 10, 100, 100),
            vx: 1.0,
            vy: 1.0,
            dir_x: true,
            dir_y: true,
            should_quit: false,
        }
    }
}

impl TuiApp for StreamingApp {
    fn event_handler(&self, _: crossterm::event::Event) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Down => {
                self.y += 1.0;
            }
            KeyCode::Up => {
                self.y -= 1.0;
            }
            KeyCode::Right => {
                self.x += 1.0;
            }
            KeyCode::Left => {
                self.x -= 1.0;
            }
            _ => {}
        }
    }

    fn tick(&mut self) {}

    fn should_quit(&self) -> bool {
        false
    }
    fn render<B: Backend>(&mut self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());
        let canvas = Canvas::default()
            .block(Block::default().borders(Borders::ALL).title("World"))
            .paint(|ctx| {
                ctx.draw(&Map {
                    color: Color::White,
                    resolution: MapResolution::High,
                });
                ctx.print(self.x, -self.y, "You are here", Color::Yellow);
            })
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0]);
        f.render_widget(canvas, chunks[0]);
        let canvas = Canvas::default()
            .block(Block::default().borders(Borders::ALL).title("Pong"))
            .paint(|ctx| {
                ctx.draw(&self.ball);
            })
            .x_bounds([10.0, 110.0])
            .y_bounds([10.0, 110.0]);
        f.render_widget(canvas, chunks[1]);
    }
}
