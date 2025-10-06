# anchor-kit

## Project Proposal

### Motivation

Given the current Rust ecosystem, developing functional, clean and well-designed graphical user interfaces (GUIs) in Rust is a very challenging and burdensome process. Immediate-mode frameworks like [egui](https://github.com/emilk/egui), have unintuitive and error-prone systems for creating layouts. This makes many aspects of GUI development, such as text and UI element alignment, require significant manual effort, resulting in long turnaround times for GUIs that more often than not, end up looking unprofessional or don’t allow for preferred functionality (see the [egui docs](https://github.com/emilk/egui?tab=readme-ov-file#disadvantages-of-immediate-mode) outlining this issue for more details). On the other hand, retained-mode GUI frameworks like [iced](https://github.com/iced-rs/iced) or [gpui](https://github.com/longbridge/gpui-component) offer more built-in structure and solve many of the issues immediate-mode frameworks have with responsive and flexible layouts, but they are often inflexible and cumbersome when integrating with existing rendering pipelines, or if aiming to add overlays on top of existing applications.

Our primary motivation then is to fill this gap in the Rust ecosystem by targeting the niche area of non-interactive GUIs for data visualization. By enabling non-interactive use-cases only, we can create an immediate-mode-only rendering system that can still respond dynamically to any changes in incoming data (size, format, etc.). This is because any changes to the data will occur between frames, and our lightweight system will allow for easy recalculation and dynamic adjustment without the need for manual readjustment and handling by the user. Our solution will allow for the simple creation of professional GUIs for data visualization and heads-up-like displays where users would not have to worry about managing any UI state across frames (which is required for retained rendering systems), while also seamlessly handling changes to incoming data without the need for any manual tweaking of their displays (which is a common problem in immediate-mode systems).


### Objective and key features

Our high-level objective is to develop a toolkit focused on creating responsive, flexible GUI layouts and overlays targeted towards non-interactive data visualization. We want to create an immediate-mode rendering system that is responsive and adaptive to changes from incoming data. This tool would help developers to streamline UI creation and reduce complexity around managing UI states between frames, and allow seamless data handling without the need for manual tweaking of UI settings such as margins, centring, and more. There aren't many good existing tools within the Rust ecosystem to accomplish this goal, and the successful creation of this framework would fill a niche that is currently vacant.

#### Key features

* [Immediate-mode](https://en.wikipedia.org/wiki/Immediate_mode_(computer_graphics)) rendering API
  * The layout is described and rendered every frame (frame-to-frame behaviour is deterministic)
  * No scene graph or complex managed GUI system in memory
 * We are targeting non-interactive data visualization cases only, so there will be no need for event handlers 
* Flexible layout system
  * 9 anchor regions for easy UI placement (see below)
  * Responsive flexbox-like grid layout system within anchor locations
* Core set of non-interactive primitives 
  * Panel
  * Text
  * Image
  * Pill/ Chip
  * Table
* Supports multiple rendering pipelines
  * Initial support will include integrations for [winit](https://docs.rs/winit/latest/winit/) and [wgpu](https://docs.rs/wgpu/latest/wgpu/)
  * Example apps for each (2 simple examples included with the repo to start)

#### Layout system idea

The general concept for the layout system of our toolkit is twofold. First, at a high level, we will provide 9 anchor points which are relative to the rendered window size. These 9 anchor points will subdivide the window equally, providing users with a simple way to “anchor” their UI elements to different portions of the screen. This system is particularly useful for heads-up and overlay-style displays, where data visualization likely wants to be rendered in the corners of the screen, so as not to cover the primary application being rendered. 

These anchor points will be laid out as follows:

||||
| :---        |    :----:   |          ---: |
|top-right|top-middle|top-left|
|middle-right|middle|middle-left|
|bottom-right|bottom-middle|bottom-left| 

The second core concept of the adjustable layout system will be the concept of flexible grids. These grids will be structured as a flexible, easy-to-understand system that allows users to arrange UI elements *within* the anchor point blocks. The grid systems themselves will provide responsiveness for the elements within them, such that general rendered layouts will remain in the same structure and maintain spacing and centring even in the case that data changes between frames (for example, a numeric data field within a pill goes from 10 -> 100,000 between frames and the margins have to be resized). The core system of these grid layouts will be heavily inspired by [Bootstrap’s grid system](https://getbootstrap.com/docs/5.0/layout/grid/).

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
