use std::{iter, sync::Arc};

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use chrono::prelude::*;

use anchor_kit_core::{
    anchor::AnchorPosition,
    primitives::color::Color,
    style::{Insets, SizingPolicy, Style, TextStyle},
};
use anchor_kit_core::{FrameInfo as UiFrameInfo, UIState};
use anchor_kit_wgpu::{Renderer, ScreenInfo as GpuFrameInfo};

// This will store the state of our app
// lib.rs

struct Data {
    speed: usize,
    rpm: usize,
    time: DateTime<Local>,
}

pub struct State {
    renderer: Renderer,
    ui_state: UIState,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    window: Arc<Window>,
    data: Data,
}

impl State {
    // We don't need this to be async right now,
    // but we will in the next tutorial
    pub async fn new(window: Arc<Window>) -> anyhow::Result<State> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                //experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let renderer = Renderer::new(&device, &queue, surface_format);

        let ui_state = UIState::new([size.width, size.height]);

        let data = Data {
            speed: 0,
            rpm: 500,
            time: Local::now(),
        };

        Ok(Self {
            renderer,
            ui_state,
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            window,
            data,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }

    fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }

    fn update(&mut self) {
        //self.window.request_redraw();

        self.data.time = Local::now();
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let ui_frame_info = UiFrameInfo {
            size: [self.config.width, self.config.height],
        };

        let render_list = self.ui_state.generate_frame(ui_frame_info, |ui| {
            ui.anchor(
                AnchorPosition::TopCenter,
                Some(Style {
                    ..Default::default()
                }),
                |ui| {
                    ui.flex_column(
                        Some(Style {
                            ..Default::default()
                        }),
                        |ui| {
                            ui.flex_row(
                                Some(Style {
                                    align_x: anchor_kit_core::style::Align::Middle,
                                    ..Default::default()
                                }),
                                |ui| {
                                    ui.pill(
                                        Some(Style {
                                            background_color: Color {
                                                r: 255,
                                                g: 255,
                                                b: 255,
                                                a: 127,
                                            },
                                            border_color: Color {
                                                r: 50,
                                                g: 50,
                                                b: 50,
                                                a: 255,
                                            },
                                            border_radius: [10.0, 10.0, 10.0, 10.0],
                                            border_width: 3.0,
                                            padding: Insets { top: 5, right: 5, bottom: 5, left: 5 },
                                            ..Default::default()
                                        }),
                                        |ui| {
                                            ui.text(
                                                self.data.time.format("%d/%b/%Y | %H:%M:%S.%3f").to_string(),
                                                Some(Style{
                                                    padding: Insets { top: 5, right: 80, bottom: 5, left: 5 },
                                                    ..Default::default()
                                                }),
                                                Some(TextStyle {
                                                    font_size: 32.0,
                                                    font_family:
                                                        anchor_kit_core::style::FontFamily::Monospace,
                                                    text_color: Color {
                                                        r: 255,
                                                        g: 0,
                                                        b: 0,
                                                        a: 255,
                                                    },
                                                    ..Default::default()
                                                }),
                                            );
                                        },
                                    );
                                },
                            );
                        },
                    );
                },
            );
            ui.anchor(
                AnchorPosition::BottomLeft,
                Some(Style {
                    ..Default::default()
                }),
                |ui| {
                    ui.flex_column(
                        Some(Style {
                            ..Default::default()
                        }),
                        |ui| {
                            ui.flex_row(
                                Some(Style {
                                    align_x: anchor_kit_core::style::Align::Middle,
                                    ..Default::default()
                                }),
                                |ui| {
                                    ui.pill(
                                        Some(Style {
                                            background_color: Color {
                                                r: 255,
                                                g: 255,
                                                b: 255,
                                                a: 127,
                                            },
                                            border_color: Color {
                                                r: 50,
                                                g: 50,
                                                b: 50,
                                                a: 255,
                                            },
                                            border_radius: [10.0, 10.0, 10.0, 10.0],
                                            border_width: 3.0,
                                            padding: Insets { top: 5, right: 5, bottom: 5, left: 5 },
                                            ..Default::default()
                                        }),
                                        |ui| {

                                        }
                                    );
                                },
                            );
                        },
                    );
                },
            );
        });

        let frame_info = GpuFrameInfo {
            size_px: [self.config.width, self.config.height],
            scale_factor: self.window.scale_factor() as f32,
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0, //0 alpha = transparent
                    }),
                    store: wgpu::StoreOp::Store,
                },
                //depth_slice: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        self.renderer.render(
            &self.device,
            &self.queue,
            &mut render_pass,
            &frame_info,
            &render_list,
        );

        drop(render_pass);

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub struct App {
    state: Option<State>,
}

impl App {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes().with_transparent(true);

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        // Allow click through, unsupported on iOS, Android, Web, X11, Orbital
        window.set_cursor_hittest(false).unwrap();

        // use pollster to await
        self.state = Some(pollster::block_on(State::new(window)).unwrap());
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfig the surface if lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = state.window.inner_size();
                        state.resize(size.width, size.height);
                    }
                    Err(e) => {
                        log::error!("Unable to Render {}", e);
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
            _ => {}
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new();

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    event_loop.run_app(&mut app)?;

    Ok(())
}
