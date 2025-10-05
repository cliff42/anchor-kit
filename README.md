# anchor-kit

## Project Proposal

### Motivation

### Objective and key features

Our high-level objective is to develop a toolkit focused on creating responsive, flexible GUI layouts and overlays targeted towards non-interactive data visualization. We want to create an immediate-mode rendering system that is responsive and adaptive to changes from incoming data. This tool would help developers to streamline UI creation and reduce complexity around managing UI states between frames, and allow seamless data handling without the need for manual tweaking of UI settings such as margins, centring, and more. There aren't many good existing tools within the Rust ecosystem to accomplish this goal, and the successful creation of this framework would fill a niche that is currently vacant.

### Tentative plan

#### repo structure (from top level)
```
crates 
  - anchor-kit-core
    - primitives
      - rectangle.rs
      - table.rs
        …
    - lib.rs (core layout functionality etc.)
  - anchor-kit-winit (winit integration)
  - anchor-kit-wpgu (wgpu integration)
examples
  - winit-example
  - wpgu-example
Cargo.toml (workspace cargo file)
README.md
... 
```
