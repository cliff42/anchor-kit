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

Above are the various options for `anchor` points within the grid system, each section you make can be broken down into a 3x3, as shown.
```
ui.anchor(<AnchorPosition>, <Style>, |closure|)

ui.anchor(AnchorPosition::TopCenter, None, |ui| {
    ui.flex_row(...
```

**Flex elements:**

<img width="575" height="453" alt="Screenshot 2025-12-15 at 2 39 28 AM" src="https://github.com/user-attachments/assets/d72fd696-9406-463f-82de-f27c69ccf88d" />

Above are various examples of ways `flex_row`s and `flex_column`s can be arraged within an `anchor` point. On top is a `flex_column` rendering multiple rows, and in the middle is a `flex_row` rendering multiple columns. At the bottom is shown how `Style` options can affect positions of these elements within, specifically showing how `alignment_y` takes affect in a `flex_row`.

```
ui.flex_row(<Style>, |closure|);
ui.flex_column(<Style>, |closure|);

ui.flex_row(None, |ui| {
    ui.text("col1".to_string() ...
    ui.text("col2".to_string() ...
```

**Pill elements:**

<img width="569" height="448" alt="Screenshot 2025-12-15 at 2 40 16 AM" src="https://github.com/user-attachments/assets/e81c6ed6-09ad-4741-a849-b2529804f0af" />

Aboce shows various permuations of the `pill` element; a basic shape provided within `anchor-kit`. This is a flexible element that is premuatable with the `Style` parameter passed in. This element is modifiable by every option within the `Style` parameter.

```
ui.pill(<Style>, |closure|);

ui.pill(
    Some(Style{
        background_color: ... ,
        border_radius: ... ,
        border_color: ... ,
        padding: ... ,
        ..Default::default()
    }),
    |ui| { ... }
);
```

**Image element:**

<img width="570" height="448" alt="Screenshot 2025-12-15 at 2 40 34 AM" src="https://github.com/user-attachments/assets/d3f7fcb4-aabc-4737-8916-4e35c4ac32bf" />

Above shows an example of rendering an image texture onto the window. Displaying these requires a few extra steps before they can be rendered onto a winodow. First, an image file has to be read in as bytes, and second, the image has to be processed using the `Renderer`s `get_image_id_from_btyes` functions. This generates a `Uuid` for the image that is then used to render it.

```
ui.image(<Image: Uuid>, <Style>);

let mut renderer = Renderer::new(...);
let diffuse_bytes = include_bytes!("example.png");
let image_uuid = renderer.get_image_id_from_bytes(<Device>, <Queue>, diffuse_bytes);
...
ui.image(
    image_uuid,
    Some(Style {
        width: anchor_kit_core::style::SizingPolicy::Fixed(400),
        height: anchor_kit_core::style::SizingPolicy::Fixed(500),
        border_radius: [40.0, 0.0, 40.0, 0.0],
        ..Default::default()
    })
);
```

**Text elements:**

<img width="573" height="448" alt="Screenshot 2025-12-15 at 2 41 00 AM" src="https://github.com/user-attachments/assets/c239dda8-bb34-476f-b426-8e4b65a333f2" />

Above shows an example of `text` rendering with various `TextStyle`s applied to them. `TextStyle` is a distinct styling paramater from `Style` that is exclusivly used for formating how text will be output, with various font options and a color setting.

```
pub struct TextStyle {
    pub font_size: f32,            // Font size
    pub line_height: f32,          // Line height
    pub font_family: FontFamily,   // Select included font family or import your own
    pub font_weight: FontWeight,   // Thin, ExtraLight, Light, Normal, Medium, SemiBold... 
    pub font_style: FontStyle,     // Normal, Italic, Bold
    pub text_color: Color,         // Text color
}

ui.text(<Text: String>, <Style>, <TextStyle>);

ui.text(
    "Hello World with Anchor-Kit!".to_string(),
    Some(Style {
        margin: Insets {
            top: 5,
            right: 0,
            bottom: 0,
            left: 0,
        },
        ..Default::default()
    }),
    Some(TextStyle {
        font_size: 16.0,
        line_height: 20.0,
        text_color: anchor_kit_core::primitives::color::Color {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        },
        font_weight: anchor_kit_core::style::FontWeight::Bold,
        ..Default::default()
    }),
);
```

**Divider elements:**

<img width="569" height="448" alt="Screenshot 2025-12-15 at 2 41 27 AM" src="https://github.com/user-attachments/assets/a67a78a0-96e4-44f1-b822-ef76854a1e5c" />

Above shows an example using the `divider` element. This element can be used to dvide up a `flex_row` or `flex_column` with lines to create visual seperation such as for a table.

```
ui.divider(<DividerOrientation>, <Thickness: u32>, <Style>);

ui.flex_column(None, |ui| {
    ui.flex_row(None, |ui| {
        ui.text("top_left".to_string(), None, None);
        ui.divider(anchor_kit_core::element::DividerOrientation::Vertical, 10, None);
        ui.text("top_right".to_string(), None, None);
    });
    ui.divider(anchor_kit_core::element::DividerOrientation::Horizontal, 10, None);
    ui.flex_row(None, |ui| {
        ui.text("bottom_left".to_string(), None, None);
        ui.divider(anchor_kit_core::element::DividerOrientation::Vertical, 10, None);
        ui.text("bottom_right".to_string(), None, None);
    });
});
```

**Overlay Example:**


https://github.com/user-attachments/assets/815acfab-0847-4df1-992b-09b16ae6940d


## Developer's Guide

anchor-kit is designed as a set of packages which developers can use to integrate directly into their existing rendering loops. The primary package is [anchor-kit-core](https://crates.io/crates/anchor-kit-core), which defines the elements, primitives and styling, handles the responsive layout and provides the easy-to-use declarative API for GUI creation. We also have an integration with wgpu ([anchor-kit-wgpu](https://crates.io/crates/anchor-kit-wgpu)), which developers can use to convert the primitives into renderable data and add it to their wgpu frame buffers.

At a high-level, to integrate anchor-kit into a wgpu rendering pipeline, there are three steps:

1. Instantiate a new `anchor_kit_wgpu::Renderer` (and register textures if required)
2. Call the `anchor_kit_core::generate_frame` function, passing in the GUI description using the declarative API to get the list of renderable primitives
3. Call the `anchor_kit_wgpu::Renderer::render()` function, passing in the generated primitives to add the data to the wgpu frame buffers

To get started with anchor-kit:

`cargo add anchor-kit-core && cargo add anchor-kit-wgpu`

_anchor_kit_wgpu::Renderer_ instantiation:

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
let mut renderer = Renderer::new(&device, &queue, surface_format);
```


## Reproducibility Guide

## Contributions by each team member

## Lessons Learned

Computer graphics is hard.
