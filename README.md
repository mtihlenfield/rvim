# rvim

Small, mostly featureless, vim implementation in Rust. It's prototype quality code and the first non-trivial thing I've written in Rust. Don't judge it too hard...

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
