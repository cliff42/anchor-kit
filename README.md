# anchor-kit

anchor-kit is a lightweight, immediate-mode UI rendering library for non-interactive overlay data visualization. It enables responsive layouts, so users don't have worry about resizing their elements manually when data changes, while still enabling a simple immediate-mode rendering framework. This is posisble since anchor-kit is purpose-built for non-interactive cases where no user input events are expected.

### Features

The core functionality behind anchor-kit is the simple API it provides for users to create GUIs using their existing graphics pipelines. In order to actual render the GUI created with anchor-kit, a `render_list` is created, which consists of anchor-kit primitives, which is then passed into the anchor-kit integration (for now just wgpu), which handles the actual rendering.

This makes creating GUIs with anchor-kit relatively simple, and it essentially boils down to two new additions to the user’s existing pipeline:

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



### Lessons Learned

Computer graphics is hard.