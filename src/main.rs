use crossterm::event;

mod gap_buf;
mod model;
mod screen;

enum EventResult {
    Exit,
    Key(),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            event::Event::Resize(cols, rows) => println!("Resized to {cols}x{rows}"),
            _ => {
                screen.update(&model)?;
            }
        };
    }

    Ok(())
}
