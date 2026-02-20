use crossterm::event;
use log::info;
use log4rs;

mod gap_buf;
mod model;
mod screen;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    info!("Starting rvim.");
    let mut screen = screen::Screen::new();
    let mut model = model::Model::new();
    screen.update(&model)?;

    loop {
        let ev = event::read()?;
        match ev {
            event::Event::Key(key_event) => {
                let should_exit = model.update(key_event);
                if should_exit {
                    break;
                }
                screen.update(&model)?;
            }
            event::Event::Resize(cols, rows) => info!("Resized to {cols}x{rows}"),
            _ => {
                screen.update(&model)?;
            }
        };
    }

    info!("Exiting rvim.");

    Ok(())
}
