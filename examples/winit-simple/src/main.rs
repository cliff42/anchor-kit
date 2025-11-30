// TODO: this doesn't have winit integration for now, just simple testing

let mut ui_state = UIState::new([800, 600]);
let frame_info = FrameInfo { size: [800, 600], time_ns: 0.0 };

let render_list = ui_state.generate_frame(frame_info, |ui| {
    ui.anchor(AnchorPosition::TopLeft, [100, 50], |ui| {
        ui.text("hello world".into());
    });
});