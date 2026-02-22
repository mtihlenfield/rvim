use crossterm::event;
use crossterm::terminal;
use log::{error, info};
use log4rs;
use std::panic;

mod gap_buf;
mod screen;
mod state;

fn main() {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    panic::set_hook(Box::new(|panic_info| {
        // TODO: consider using a crate for this? It doesn't work very well.
        let (filename, line) = panic_info
            .location()
            .map(|loc| (loc.file(), loc.line()))
            .unwrap_or(("<unknown>", 0));

        let cause = panic_info
            .payload()
            .downcast_ref::<&str>()
            .unwrap_or(&"<cause unknown>");

        error!("A panic occurred at {}:{}: {}", filename, line, cause);
    }));

    info!("Starting rvim.");
    let (cols, rows) = terminal::size().expect("Failed to get term size.");
    let mut screen = screen::Screen::new(rows, cols);
    let mut state = state::EditorState::new(rows, cols);
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
                // TODO: resizing not working correctly - leaving behind artifacts
                screen.resize(cols, rows);
                if let Err(err) = screen.update(&state) {
                    error!("Got error while updating screen: {err}");
                }
            }
            _ => {}
        };
    }

    info!("Exiting rvim.");
}
