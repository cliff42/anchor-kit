use std::{iter, sync::Arc};

use uuid::Uuid;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use anchor_kit_core::{
    anchor::AnchorPosition,
    style::{Insets, SizingPolicy, Style, TextStyle},
};
use anchor_kit_core::{FrameInfo, UIState};
use anchor_kit_wgpu::{Renderer, ScreenInfo};

pub struct State {
    renderer: Renderer,
    ui_state: UIState,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    window: Arc<Window>,
    image_id: Uuid, // just one image id here for the example, in reality these should be managed better
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<State> {
        let size = window.inner_size();

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
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
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

        let mut renderer = Renderer::new(&device, &queue, surface_format);

        let ui_state = UIState::new([size.width, size.height]);

        // for image rendering
        let diffuse_bytes = include_bytes!("test.png");
        let image_id = renderer.get_image_id_from_bytes(&device, &queue, diffuse_bytes);

        Ok(Self {
            renderer,
            ui_state,
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            window,
            image_id,
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
        //TODO
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

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

        let ui_frame_info = FrameInfo {
            size: [self.config.width, self.config.height],
        };

        // HERE IS WHERE anchor-kit GUIS ARE CREATED (UPDATE THIS RENDER LIST GENERATION)
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
            ui.anchor(
                AnchorPosition::TopCenter,
                Some(Style {
                    width: SizingPolicy::Fixed(400),
                    height: SizingPolicy::Fixed(600),
                    ..Default::default()
                }),
                |ui| {
                    ui.flex_column(
                        Some(Style {
                            width: SizingPolicy::FillParent,
                            height: SizingPolicy::FillParent,
                            justify_y: anchor_kit_core::style::Align::Start,
                            ..Default::default()
                        }),
                        |ui| {
                            ui.pill(
                                Some(Style {
                                    width: SizingPolicy::Fixed(80),
                                    height: SizingPolicy::Fixed(30),
                                    background_color: anchor_kit_core::primitives::color::Color {
                                        r: 255,
                                        g: 0,
                                        b: 0,
                                        a: 255,
                                    },
                                    border_radius: [15.0, 15.0, 15.0, 15.0],
                                    justify_x: anchor_kit_core::style::Align::Middle,
                                    justify_y: anchor_kit_core::style::Align::Middle,
                                    align_x: anchor_kit_core::style::Align::Middle,
                                    margin: Insets {
                                        top: 10,
                                        right: 10,
                                        bottom: 10,
                                        left: 0,
                                    },
                                    ..Default::default()
                                }),
                                |ui| {
                                    ui.text(
                                        "row 1".to_string(),
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
                                },
                            );
                            ui.pill(
                                Some(Style {
                                    width: SizingPolicy::Fixed(80),
                                    height: SizingPolicy::Fixed(30),
                                    background_color: anchor_kit_core::primitives::color::Color {
                                        r: 0,
                                        g: 255,
                                        b: 0,
                                        a: 255,
                                    },
                                    border_radius: [15.0, 15.0, 15.0, 15.0],
                                    justify_x: anchor_kit_core::style::Align::Middle,
                                    justify_y: anchor_kit_core::style::Align::Middle,
                                    align_x: anchor_kit_core::style::Align::Middle,
                                    margin: Insets {
                                        top: 10,
                                        right: 10,
                                        bottom: 10,
                                        left: 0,
                                    },
                                    ..Default::default()
                                }),
                                |ui| {
                                    ui.text(
                                        "row 2".to_string(),
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
                                },
                            );
                            ui.pill(
                                Some(Style {
                                    width: SizingPolicy::Fixed(80),
                                    height: SizingPolicy::Fixed(30),
                                    background_color: anchor_kit_core::primitives::color::Color {
                                        r: 0,
                                        g: 0,
                                        b: 255,
                                        a: 255,
                                    },
                                    border_radius: [15.0, 15.0, 15.0, 15.0],
                                    justify_x: anchor_kit_core::style::Align::Middle,
                                    justify_y: anchor_kit_core::style::Align::Middle,
                                    align_x: anchor_kit_core::style::Align::Middle,
                                    margin: Insets {
                                        top: 10,
                                        right: 10,
                                        bottom: 10,
                                        left: 0,
                                    },
                                    ..Default::default()
                                }),
                                |ui| {
                                    ui.text(
                                        "row 3".to_string(),
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
                                },
                            );
                        },
                    );
                },
            );
            ui.anchor(AnchorPosition::MiddleCenter, None, |ui| {
                ui.flex_row(None, |ui| {
                    ui.pill(
                        Some(Style {
                            width: SizingPolicy::Fixed(80),
                            height: SizingPolicy::Fixed(30),
                            background_color: anchor_kit_core::primitives::color::Color {
                                r: 100,
                                g: 100,
                                b: 0,
                                a: 255,
                            },
                            border_radius: [15.0, 15.0, 15.0, 15.0],
                            justify_x: anchor_kit_core::style::Align::Middle,
                            justify_y: anchor_kit_core::style::Align::Middle,
                            align_x: anchor_kit_core::style::Align::Middle,
                            margin: Insets {
                                top: 10,
                                right: 10,
                                bottom: 10,
                                left: 0,
                            },
                            ..Default::default()
                        }),
                        |ui| {
                            ui.text(
                                "col 1".to_string(),
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
                        },
                    );
                    ui.pill(
                        Some(Style {
                            width: SizingPolicy::Fixed(80),
                            height: SizingPolicy::Fixed(30),
                            background_color: anchor_kit_core::primitives::color::Color {
                                r: 0,
                                g: 100,
                                b: 100,
                                a: 255,
                            },
                            border_radius: [15.0, 15.0, 15.0, 15.0],
                            justify_x: anchor_kit_core::style::Align::Middle,
                            justify_y: anchor_kit_core::style::Align::Middle,
                            align_x: anchor_kit_core::style::Align::Middle,
                            margin: Insets {
                                top: 10,
                                right: 10,
                                bottom: 10,
                                left: 0,
                            },
                            ..Default::default()
                        }),
                        |ui| {
                            ui.text(
                                "col 2".to_string(),
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
                        },
                    );
                    ui.pill(
                        Some(Style {
                            width: SizingPolicy::Fixed(80),
                            height: SizingPolicy::Fixed(30),
                            background_color: anchor_kit_core::primitives::color::Color {
                                r: 100,
                                g: 0,
                                b: 100,
                                a: 255,
                            },
                            border_radius: [15.0, 15.0, 15.0, 15.0],
                            justify_x: anchor_kit_core::style::Align::Middle,
                            justify_y: anchor_kit_core::style::Align::Middle,
                            align_x: anchor_kit_core::style::Align::Middle,
                            margin: Insets {
                                top: 10,
                                right: 10,
                                bottom: 10,
                                left: 0,
                            },
                            ..Default::default()
                        }),
                        |ui| {
                            ui.text(
                                "col 3".to_string(),
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
                        },
                    );
                });
            });
            ui.anchor(
                anchor_kit_core::anchor::AnchorPosition::BottomCenter,
                None,
                |ui| {
                    ui.flex_row(
                        Some(Style {
                            justify_x: anchor_kit_core::style::Align::End,
                            width: anchor_kit_core::style::SizingPolicy::Fixed(800),
                            height: anchor_kit_core::style::SizingPolicy::Fixed(150),
                            ..Default::default()
                        }),
                        |ui| {
                            ui.pill(
                                Some(Style {
                                    background_color: anchor_kit_core::primitives::color::Color {
                                        r: 100,
                                        g: 50,
                                        b: 50,
                                        a: 255,
                                    },
                                    border_radius: [20.0, 20.0, 20.0, 15.0],
                                    justify_x: anchor_kit_core::style::Align::Middle,
                                    justify_y: anchor_kit_core::style::Align::Middle,
                                    align_y: anchor_kit_core::style::Align::Start,
                                    margin: Insets {
                                        top: 10,
                                        right: 10,
                                        bottom: 10,
                                        left: 0,
                                    },
                                    padding: Insets {
                                        top: 5,
                                        right: 20,
                                        bottom: 10,
                                        left: 20,
                                    },
                                    ..Default::default()
                                }),
                                |ui| {
                                    ui.text(
                                        "justify end - align start".to_string(),
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
                                },
                            );
                            ui.pill(
                                Some(Style {
                                    background_color: anchor_kit_core::primitives::color::Color {
                                        r: 0,
                                        g: 100,
                                        b: 100,
                                        a: 255,
                                    },
                                    border_radius: [20.0, 20.0, 20.0, 20.0],
                                    justify_x: anchor_kit_core::style::Align::Middle,
                                    justify_y: anchor_kit_core::style::Align::Middle,
                                    align_y: anchor_kit_core::style::Align::Middle,
                                    margin: Insets {
                                        top: 10,
                                        right: 10,
                                        bottom: 10,
                                        left: 0,
                                    },
                                    padding: Insets {
                                        top: 5,
                                        right: 20,
                                        bottom: 10,
                                        left: 20,
                                    },
                                    ..Default::default()
                                }),
                                |ui| {
                                    ui.text(
                                        "justify end - align middle".to_string(),
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
                                },
                            );
                            ui.pill(
                                Some(Style {
                                    background_color: anchor_kit_core::primitives::color::Color {
                                        r: 100,
                                        g: 0,
                                        b: 100,
                                        a: 255,
                                    },
                                    border_radius: [20.0, 20.0, 20.0, 20.0],
                                    justify_x: anchor_kit_core::style::Align::Middle,
                                    justify_y: anchor_kit_core::style::Align::Middle,
                                    align_y: anchor_kit_core::style::Align::End,
                                    margin: Insets {
                                        top: 10,
                                        right: 10,
                                        bottom: 10,
                                        left: 0,
                                    },
                                    padding: Insets {
                                        top: 5,
                                        right: 20,
                                        bottom: 10,
                                        left: 20,
                                    },
                                    ..Default::default()
                                }),
                                |ui| {
                                    ui.text(
                                        "justify end - align end".to_string(),
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
                                },
                            );
                        },
                    );
                },
            );
        });

        let screen_info = ScreenInfo {
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
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        self.renderer.render(
            &self.device,
            &self.queue,
            &mut render_pass,
            &screen_info,
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
        let mut window_attributes = Window::default_attributes();

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

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

    event_loop.run_app(&mut app)?;

    Ok(())
}
