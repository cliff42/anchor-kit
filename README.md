# anchor-kit

## Project Proposal

### Motivation

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

The second core concept of the adjustable layout system will be the concept of flexible grids. These grids will be structured as a flexible, easy-to-understand system that allows users to arrange UI elements within the anchor point blocks. The grid systems themselves will provide responsiveness for the elements within them, such that general rendered layouts will remain in the same structure and maintain spacing and centring even in the case that data changes between frames (for example, a numeric data field within a pill goes from 10 -> 100,000 between frames and the margins have to be resized). The core system of these grid layouts will be heavily inspired by [Bootstrap’s grid system](https://getbootstrap.com/docs/5.0/layout/grid/).

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
