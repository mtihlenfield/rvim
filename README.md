# rvim

Small, mostly featureless, vim implementation in Rust. Just built for learning Rust.

## Notes

- Implement the basics
    - Single window, single buffer
    - Statusline (bottom)
        - mode indicator
        - position indicator
        - information (e.g. can't save because file is not writeable)
    - Modes: Normal, Insert, Command (:)
    - Motions: h, j, k l

- If I have time:
    - Visual mode
    - Signcolumn (left bar)
        - line numbers?
        - signs?
    - undo/redo
    - copy/paste
    - async?


- Probably use https://github.com/crossterm-rs/crossterm
- Could use a Rope or Gap Buffer: https://en.wikipedia.org/wiki/Rope_(data_structure)
- Some other design patterns:
    - https://en.wikipedia.org/wiki/Memento_pattern
    - https://en.wikipedia.org/wiki/Command_pattern
    - Model View Update (TEA)
        - Model = buffer + mode + cursor position?
            - is buffer window position (what part of the file we're showing) part of the model?
        - Top level View
            - child Window (only one for now)
                - child: statusline
                - child: buffer
                    - child: cursor - not sure if this should be here or not. 
                    - note: buffer can only show one screen's worth of text. A movement in normal mode will result in an update to the buffer View even if the text content doesn't change.
        - Update = get command, update model, update view based on model

- How to handle events?
    - Types:
        - insert/delete
        - motion
        - mode change
        - command line command
    - Mode dependent:
        - Normal: Commands are all for navigation, except `:`, `i`, `a` which change modes.
        - Insert mode: All input is just insert.
        - Command Mode: 
    - All modes (except normal) switch back to normal on escape key

- Going to need to handle signals. Does crossterm help with this?

- Main loop listens for events, updates the model, and then passes that to the view


