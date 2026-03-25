use crossterm::event;
use crossterm::terminal;
use log::{error, info};
use log4rs;
use std::env;

mod buffer;
mod char_iter;
mod gap_buf;
mod line_iter;
mod position;
mod screen;
mod slice;
mod state;

fn main() {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    log_panics::init();
    info!("Starting rvim.");

    let mut args = env::args();
    args.next();

    let mut state = state::EditorState::new();
    if let Some(path) = args.next() {
        if let Err(err) = state.open_file(&path) {
            let msg = format!("Could not open file '{}': {}", path, err);
            error!("{}", msg);

            return;
        }
    }

    let (cols, rows) = terminal::size().expect("Failed to get term size.");
    info!("Term size - rows: {}, cols: {}", rows, cols);
    let mut screen = screen::Screen::new(rows, cols);
    screen.update(&state).expect("Failed to init screen.");

    loop {
        let ev = event::read().expect("Failed to read event.");
        match ev {
            event::Event::Key(key_event) => {
                let should_exit = state.update(key_event);
                if should_exit {
                    break;
                }
                if let Err(err) = screen.update(&state) {
                    error!("Got error while updating screen: {err}");
                }
            }
            event::Event::Resize(cols, rows) => {
                if let Err(err) = screen.resize(rows, cols) {
                    error!("Got error while resizing screen: {err}");
                }

                if let Err(err) = screen.update(&state) {
                    error!("Got error while updating screen: {err}");
                }
            }
            _ => {}
        };
    }

    info!("Exiting rvim.");
}
