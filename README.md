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
    - Use u8 as underlying type in gap buffer instead of char to improve memory
    footprint
    - Line cache/storage for faster line access
    - Correct handling of tabs (\t)
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

### Insert Mode Delete scenarios:

- Current pos: 
    - start of line
        - Need to delete the \n and move up to end of the next line
            - How do I determine what the new global offset is?
                - It's just current - 1, unless the char you just deleted was the 
                the last char and was a newline
    - middle of line
        - just delete the char and move left 1
    - end of line
        - just delete the char and move left 1

### Viewport Moving

- Need an anchor position that is not the cursor
- Should probably the top left of the viewport
    - So would be (0,0) in viewport terms, but somewhere in global terms
- in insert mode:
    - if the global cursor row is > anchor_row + viewport_rows, move the viewport down one
    - if the global cursor row is <  anchor_row, move the viewport up one
- Does the buffer need to track the viewport if we do it this way? Anchor pos should be global, and then viewport cursor pos can be determined relative to the anchor
