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
    let buffer = model::Buffer {
        buf: "hello, world".to_string(),
        cursor_position: model::Position { col: 0, row: 0 },
    };
    let model = model::Model::new(buffer);
    screen.update(&model)?;

    loop {
        let ev = event::read()?;
        match ev {
            event::Event::Key(key_event) => {
                if key_event.code == event::KeyCode::Char('q') {
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
