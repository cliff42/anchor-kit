# anchor-kit

anchor-kit is a lightweight, immediate-mode UI rendering library for non-interactive overlay data visualization. It enables responsive layouts, so users don't have worry about resizing their elements manually when data changes, while still enabling a simple immediate-mode rendering framework. This is posisble since anchor-kit is purpose-built for non-interactive cases where no user input events are expected.

`cargo add anchor-kit-core && cargo add anchor-kit-wgpu`

anchor-kit-core [crate](https://crates.io/crates/anchor-kit-core)

anchor-kit-wgpu [crate](https://crates.io/crates/anchor-kit-wgpu)

# Video Slide Presentation

Slides: [here](https://docs.google.com/presentation/d/1OnSfOBHfAmb4jpYNjY7fyvvHU6kqUAGf0jqdkyfNM4M/edit?usp=sharing).

# Video Demo

# Final Report

## Team members

| Team member | Email                            | Student Number |
| :-----------| :------------------------------: | ---------: |
| Chris Cliff |   chris.cliff@mail.utoronto.ca   | 1012787085 |
| Piotr Nowak |   piotr.nowak@mail.utoronto.ca   |            |


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

## Reproducibility Guide

## Contributions by each team member

## Lessons Learned

Computer graphics is hard.
