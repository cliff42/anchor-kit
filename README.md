# anchor-kit

anchor-kit is a lightweight, immediate-mode UI rendering library for non-interactive overlay data visualization. It enables responsive layouts, so users don't have to worry about resizing their elements manually when data changes, while still enabling a simple, immediate-mode rendering framework. This is possible since anchor-kit is purpose-built for non-interactive cases where no user input events are expected.

`cargo add anchor-kit-core && cargo add anchor-kit-wgpu`

anchor-kit-core [crate](https://crates.io/crates/anchor-kit-core)

anchor-kit-wgpu [crate](https://crates.io/crates/anchor-kit-wgpu)

# Video Slide Presentation

https://www.youtube.com/watch?v=cf9ffnPPhQ8

Slides: [here](https://docs.google.com/presentation/d/1OnSfOBHfAmb4jpYNjY7fyvvHU6kqUAGf0jqdkyfNM4M/edit?usp=sharing).

# Video Demo

https://www.youtube.com/watch?v=7skxVpKRIko

# Final Report

## Team members

| Team member | Email                            | Student Number |
| :-----------| :------------------------------: | ---------: |
| Chris Cliff |   chris.cliff@mail.utoronto.ca   | 1012787085 |
| Piotr Nowak |   piotr.nowak@mail.utoronto.ca   | 1012752148 |


## Motivation

## Objectives

## Features

### Core libraries used
- [wgpu](https://docs.rs/wgpu/latest/wgpu/)
- [winit](https://docs.rs/winit/latest/winit/) (for windowed rendering)
- [glyphon](https://github.com/grovesNL/glyphon) (text rendering in wgpu)

<img width="775" height="303" alt="Screenshot 2025-12-15 at 3 10 16 AM" src="https://github.com/user-attachments/assets/c093b2d2-a1ef-4a10-a635-8248fb4b5804" />


The core functionality behind anchor-kit is the simple API it provides for users to create GUIs using their existing graphics pipelines. In order to actually render the GUI created with anchor-kit, a `render_list` is created, which consists of anchor-kit primitives, which is then passed into the anchor-kit integration (for now, just wgpu), which handles the actual rendering.

This makes creating GUIs with anchor-kit relatively simple, and it essentially boils down to two new additions to the user’s existing pipeline:

Example:

```
... (wgpu/ winit setup)

let render_list = self.ui_state.generate_frame(ui_frame_info, |ui| {
    ui.anchor(AnchorPosition::BottomLeft, None, |ui| {
        ui.image(
            self.image_id,
            Some(Style {
                width: anchor_kit_core::style::SizingPolicy::Fixed(400),
                height: anchor_kit_core::style::SizingPolicy::Fixed(500),
                ..Default::default()
            }),
        );
    });
});

anchor_kit_wgpu_renderer.render(
            &self.device,
            &self.queue,
            &mut render_pass,
            &screen_info,
            &render_list,
);

... additional rendering if required

self.queue.submit(iter::once(encoder.finish())); // submit everything to be rendered by wgpu (including the anchor-kit pass)
```

**Styling:**

```
pub struct Style {
    pub padding: Insets, // top, right, bottom, left padding for the element 
    pub margin: Insets, // top, right, bottom, left margin for the element
    pub width: SizingPolicy, // fixed, fill or automatic (based on element content) sizing 
    pub height: SizingPolicy, // fixed, fill or automatic (based on element content) sizing 
    pub align_x: Align, // element x alignment within parent element (start, middle, end)
    pub align_y: Align, // element y alignment within parent element (start, middle, end)
    pub justify_x: Align, // x alignment of content within the element (start, middle, end)
    pub justify_y: Align, // y alignment of content within the element (start, middle, end)
    pub background_color: Color, // background color for the element (red, green, blue, alpha)
    pub border_color: Color, // border color for the element (red, green, blue, alpha)
    pub border_radius: [f32; 4], // radius for element corner rounding (top-left, top-right, bottom-right, bottom-left)
    pub border_width: f32, // size of the element’s border
}
```

**Anchor positions:**

<img width="573" height="449" alt="Screenshot 2025-12-15 at 2 39 05 AM" src="https://github.com/user-attachments/assets/4d92d418-e021-4fd9-a0be-926976b7ba01" />

**Flex elements:**

<img width="575" height="453" alt="Screenshot 2025-12-15 at 2 39 28 AM" src="https://github.com/user-attachments/assets/d72fd696-9406-463f-82de-f27c69ccf88d" />

**Pill elements:**

<img width="569" height="448" alt="Screenshot 2025-12-15 at 2 40 16 AM" src="https://github.com/user-attachments/assets/e81c6ed6-09ad-4741-a849-b2529804f0af" />

**Image element:**

<img width="570" height="448" alt="Screenshot 2025-12-15 at 2 40 34 AM" src="https://github.com/user-attachments/assets/d3f7fcb4-aabc-4737-8916-4e35c4ac32bf" />

**Text elements:**

<img width="573" height="448" alt="Screenshot 2025-12-15 at 2 41 00 AM" src="https://github.com/user-attachments/assets/c239dda8-bb34-476f-b426-8e4b65a333f2" />

**Divider elements:**

<img width="569" height="448" alt="Screenshot 2025-12-15 at 2 41 27 AM" src="https://github.com/user-attachments/assets/a67a78a0-96e4-44f1-b822-ef76854a1e5c" />

**Overlay Example:**


https://github.com/user-attachments/assets/815acfab-0847-4df1-992b-09b16ae6940d


## Developer's Guide

anchor-kit is designed as a set of packages which developers can use to integrate directly into their existing rendering loops. The primary package is [anchor-kit-core](https://crates.io/crates/anchor-kit-core), which defines the elements, primitives and styling, handles the responsive layout and provides the easy-to-use declarative API for GUI creation. We also have an integration with wgpu ([anchor-kit-wgpu](https://crates.io/crates/anchor-kit-wgpu)), which developers can use to convert the primitives into renderable data and add it to their wgpu frame buffers.

At a high-level, to integrate anchor-kit into a wgpu rendering pipeline, there are three steps:

1. Instantiate a new `anchor_kit_wgpu::Renderer` (and register textures if required)
2. Call the `anchor_kit_core::generate_frame` function, passing in the GUI description using the declarative API to get the list of renderable primitives
3. Call the `anchor_kit_wgpu::Renderer::render()` function, passing in the generated primitives to add the data to the wgpu frame buffers

**To get started with anchor-kit:**

`cargo add anchor-kit-core && cargo add anchor-kit-wgpu`

**_anchor_kit_wgpu::Renderer_ instantiation:**

```
... (wgpu boilerplate setup)

// get the wgpu device and queue (also boilerplate, not anchor-kit specific)
let (device, queue) = adapter
    .request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: Default::default(),
        trace: wgpu::Trace::Off,
    })
    .await?;

// wgpu surface format boilerplate (not anchor-kit specific)
let surface_caps = surface.get_capabilities(&adapter);
let surface_format = surface_caps
    .formats
    .iter()
    .find(|f| f.is_srgb())
    .copied()
    .unwrap_or(surface_caps.formats[0]);

// instantiate the anchor_kit_wgpu::Renderer using these wgpu objects
let mut anchor_kit_wgpu_renderer = Renderer::new(&device, &queue, surface_format);
```

**Register any textures (if you want to render images):**

```
let diffuse_bytes = include_bytes!("test.png");
let image_id = renderer.get_image_id_from_bytes(&device, &queue, diffuse_bytes); // store the image id somewhere in rendering state to use it during the `generate_frame()` function
```

**Use anchor-kit-core's declarative API to define GUI (run each frame):**

See examples above in the feature section for more specific element implementation details.

```
let render_list = self.ui_state.generate_frame(ui_frame_info, |ui| {
    ui.anchor(AnchorPosition::TopCenter, None, |ui| {
        ... // additional elements defined here (see more examples above in the features section)
    });
}
```

**Convert anchor-kit primitives to wgpu frame buffers, and render them (run each frame):**

```
// call the anchor_kit_wgpu_renderer `render()` function with the render_list created by the `generate_frame` function above
self.anchor_kit_wgpu_renderer.render(
    &self.device,
    &self.queue,
    &mut render_pass,
    &screen_info,
    &render_list,
);

// more wgpu boilerplate (not anchor-kit specific), but this is how the frame is actually triggered for rendering (with the frame buffers the `render()` function defines above)
drop(render_pass);
self.queue.submit(iter::once(encoder.finish()));
```

## Reproducibility Guide

Since creating a new wgpu & winit app from scratch requires a lot of boilerplate code, the easiest way to reproduce the results we demonstrated above is to use the provided example code in the `anchor-kit` repository (see `examples` dir in this repo), which already contains all of the required boilerplate code to get a window to visualize.

**Clone the repo:**

`git clone git@github.com:cliff42/anchor-kit.git`

**cd into the examples dir for the example you want to run:**

For the overlay example with dynamic data that changes per frame:

`cd anchor-kit/examples/winit-overlay`

For the simple example with static data:

`cd anchor-kit/examples/winit-simple`

**Run the example:**

(In either example dir):

`cargo run`

Once the project has finished building, a window should appear with the example GUI.

To stop the rendering, either close the window or kill the program (CTRL + C).

**Modify the example code to create your own GUIs:**

Find the `generate_frame()` function call in the example code in `lib.rs` (look for the comment: `// HERE IS WHERE anchor-kit GUIS ARE CREATED (UPDATE THIS RENDER LIST GENERATION)`).

overlay example generate_frame call: https://github.com/cliff42/anchor-kit/blob/main/examples/winit-overlay/src/lib.rs#L171

simple example generate_frame call: https://github.com/cliff42/anchor-kit/blob/main/examples/winit-simple/src/lib.rs#L143

Use the examples above in the features section and in the demo/ presentation videos to modify the GUI creation by using anchor-kit's declarative API, and modify the closure functions to create whatever GUI you wish to visualize.

**Some things to note:**

- If you want to render images, make sure to register the texture with your `anchor_kit_wgpu_renderer` and keep track of the generated texture id to pass into the image elements. See example: [here](https://github.com/cliff42/anchor-kit/blob/main/examples/winit-simple/src/lib.rs#L83-L85).

- Not all styling is applied to each element. If you are struggling to see styling changes that you apply being rendered, remember that not all style parameters have effects on every element type. For example, adding a `background-color` to text elements directly will not create a background highlight behind the text, instead you would have to create a pill element with your preferred `background-color` and wrap the text element in the pill element's closure function (the text element needs to be a child of pill element).


## Contributions by each team member

## Lessons Learned

Computer graphics and rendering are quite hard. Going into the project, we didn't fully understand all of the intricacies involved in creating a rendering library, even one with a relatively limited scope like anchor-kit. We didn't expect the amount of work required to get simple things like rounded corners or texture rendering. It was especially interesting to learn about shader development, which is something that we hadn't done before. Since we were new to shader development, it was very fun, albeit time-consuming, to learn about to differences between the types of shaders, how to hook them up to rendering pipelines, and all of the math involved in various coordinate systems and concepts like signed distance functions. We are very grateful for Inigo Quilez and the vast library of [open source shader development tutorials](https://iquilezles.org/articles/distfunctions2d/) he has published. His tutorials came in handy and were greatly appreciated when dealing with late-night shader debugging.

Another lesson learned early on in anchor-kit's development was the importance of separating concerns with regard to our core layout framework. Initially, we aimed to convert user-defined elements (via their closure functions) to renderable primitives in a _single pass_. Trying to measure elements, determine their layouts and actually convert them to primitives all at the same time proved to be very messy and led to a plethora of issues in anchor-kit's early development. After taking a step back, and doing some deep reading on best practices with render trees, we ended up determining that the best course of action was to separate the concerns into our three core passes (measure, layout, and render), which greatly reduced the complexity of our layout system and made rendering elements significantly easier. With this, we learned the importance of taking a step back to think about first principles and getting a better understanding of best practices before diving deep into actual development. We also gained a much better understanding of just what it takes to actually enable responsive elements and complex styling, which we often take for granted, especially when developing web applications. 

## Future work
- We plan to add new element integrations to `anchor-kit-core` to enable more complex data visualizations and provide users with even simpler APIs to create their desired GUIs.
- We also want to add new rendering library integrations, likely starting with a dedicated library for `winit`, which should help reduce the amount of boilerplate the user needs to add to get a GUI rendering in a window.
