use anchor_kit_core::{primitives::rectangle::Rectangle, render::RenderList};
use glyphon::{
    Attrs, Cache, Family, FontSystem, Metrics, Shaping, SwashCache, TextArea, TextAtlas,
    TextBounds, TextRenderer, Viewport,
};
use wgpu::include_wgsl;

pub struct ScreenInfo {
    pub size_px: [u32; 2], // w, h
    pub scale_factor: f32, // used by glyphon to scale text
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::NoUninit)] // TODO: do we need to go back to bytemuck POD/ zeroable here instead?
struct Vertex {
    position: [f32; 2], // x, y (normalized)
    color: [f32; 4],    // r, g, b, a
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4]; // location 0 is normalized position, location 1 is colour

    fn capacity_to_bytes(capacity: usize) -> wgpu::BufferAddress {
        (capacity * std::mem::size_of::<Self>()) as wgpu::BufferAddress
    }

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

fn get_vertex_buffer(device: &wgpu::Device, capacity_bytes: wgpu::BufferAddress) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("anchor-kit vertex buffer"),
        size: capacity_bytes,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn get_index_buffer(device: &wgpu::Device, capacity_bytes: wgpu::BufferAddress) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("anchor-kit index buffer"),
        size: capacity_bytes,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

// rectangles are made up of 4 vertices and 6 indices
// v0--v1
// |  /|
// | / |
// |/  |
// v3--v2
fn get_vertices_and_indices_for_rectangle(
    rect: &Rectangle,
    screen_info: &ScreenInfo,
    vertex_offset: u32,
) -> ([Vertex; 4], [u32; 6]) {
    let [x, y] = rect.position;
    let [w, h] = rect.size;
    let [screen_w, screen_h] = screen_info.size_px;

    // normalize pixel values
    let x0 = x as f32 / screen_w as f32;
    let x1 = (x + w) as f32 / screen_w as f32;
    let y0 = y as f32 / screen_h as f32;
    let y1 = (y + h) as f32 / screen_h as f32;

    let color = rect.color.to_rgba_f32();

    let v0 = Vertex {
        position: [x0, y0],
        color,
    };
    let v1 = Vertex {
        position: [x1, y0],
        color,
    };
    let v2 = Vertex {
        position: [x1, y1],
        color,
    };
    let v3 = Vertex {
        position: [x0, y1],
        color,
    };

    let vertices = [v0, v1, v2, v3];

    // triangles are v0 -> v2 -> v1, and v0 -> v3 -> v2. (have to go in ccw order)
    let indices = [
        vertex_offset,
        vertex_offset + 2,
        vertex_offset + 1,
        vertex_offset,
        vertex_offset + 3,
        vertex_offset + 2,
    ];

    (vertices, indices)
}

struct GlyphonRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    text_renderer: TextRenderer,
}

impl GlyphonRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        // set up glyphon text rendering (inspired by: https://github.com/grovesNL/glyphon/blob/main/examples/hello-world.rs)
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let glyphon_cache = Cache::new(device);
        let viewport = Viewport::new(device, &glyphon_cache);
        let mut atlas = TextAtlas::new(device, queue, &glyphon_cache, texture_format);
        let text_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);

        GlyphonRenderer {
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
        }
    }

    pub fn render_text(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
        screen_info: &ScreenInfo,
        render_list: &RenderList,
    ) {
        // skip rendering if no text is requested
        if render_list.text.is_empty() {
            return;
        }

        let [screen_w, screen_h] = screen_info.size_px;

        self.viewport.update(
            queue,
            glyphon::Resolution {
                width: screen_w,
                height: screen_h,
            },
        );

        let physical_width = screen_w as f32 * screen_info.scale_factor;
        let physical_height = screen_h as f32 * screen_info.scale_factor;

        // each text item is rendered to its own `TextArea` in glyphon
        let mut text_areas: Vec<TextArea> = Vec::with_capacity(render_list.text.len());

        // we need to store the buffers so they have the same lifetime as the areas (areas use the buffers)
        let mut text_buffers: Vec<glyphon::Buffer> = Vec::with_capacity(render_list.text.len());

        for text_item in &render_list.text {
            // TODO: metrics should be set by text style passed in by user
            let mut text_buffer =
                glyphon::Buffer::new(&mut self.font_system, Metrics::new(12.0, 16.0));

            text_buffer.set_size(
                &mut self.font_system,
                Some(physical_width),
                Some(physical_height),
            );

            // TODO: these styling options should be set by the user as well
            text_buffer.set_text(
                &mut self.font_system,
                &text_item.text,
                &Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );

            // TODO: should we also set things like text wrap (should this be set in style)?
            text_buffer.shape_until_scroll(&mut self.font_system, false);

            text_buffers.push(text_buffer);
        }

        for (i, text_buffer) in text_buffers.iter().enumerate() {
            let text_item = &render_list.text[i];

            let [x, y] = text_item.position;
            let [w, h] = text_item.size;
            let text_bounds = TextBounds {
                left: x as i32,
                top: y as i32,
                right: (x + w) as i32,
                bottom: (y + h) as i32,
            };

            let text_color = glyphon::Color::rgba(
                text_item.color.r,
                text_item.color.g,
                text_item.color.b,
                text_item.color.a,
            );

            text_areas.push(TextArea {
                buffer: &text_buffer,
                left: x as f32,
                top: y as f32,
                scale: screen_info.scale_factor,
                bounds: text_bounds,
                default_color: text_color,
                custom_glyphs: &[],
            });
        }

        if let Err(err) = self.text_renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            text_areas,
            &mut self.swash_cache,
        ) {
            // TODO: add better error handling
            println!("error with glyphon text prepare: {:?}", err);
            return;
        }

        if let Err(err) = self
            .text_renderer
            .render(&mut self.atlas, &self.viewport, render_pass)
        {
            // TODO: add better error handling
            println!("error with glyphon text render: {:?}", err);
            return;
        }

        self.atlas.trim();
    }
}

pub struct Renderer {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_buffer_capacity: usize,
    index_buffer: wgpu::Buffer,
    index_buffer_capacity: usize,
    glyphon_renderer: GlyphonRenderer,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        // inspired by: https://sotrh.github.io/learn-wgpu/beginner/tutorial3-pipeline/#how-do-we-use-the-shaders
        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        let initial_vertex_buffer_capacity = 1024; // reasonable estiamte for small scale applications so we don't have to resize right away
        let vertex_buffer = get_vertex_buffer(
            device,
            Vertex::capacity_to_bytes(initial_vertex_buffer_capacity),
        );

        // we want to use indexed rendering for better performance
        let initial_index_buffer_capacity = 2048;
        let index_buffer = get_index_buffer(
            device,
            (initial_index_buffer_capacity * std::mem::size_of::<u32>()) as wgpu::BufferAddress, // use u32 for size of index
        );

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("anchor-kit pipeline layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("anchor-kit render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()], // get the buffer layout description from the vertex impl
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Renderer {
            render_pipeline,
            vertex_buffer,
            vertex_buffer_capacity: initial_vertex_buffer_capacity as usize,
            index_buffer,
            index_buffer_capacity: initial_index_buffer_capacity as usize,
            glyphon_renderer: GlyphonRenderer::new(device, queue, texture_format),
        }
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
        screen_info: &ScreenInfo,
        render_list: &RenderList,
    ) {
        let mut vertices: Vec<Vertex> = vec![];
        let mut indices: Vec<u32> = vec![];

        // convert all primatives to vertices
        for rect in &render_list.rectangles {
            // offset will increment as new vertices are added
            let (new_vertices, new_indices) =
                get_vertices_and_indices_for_rectangle(rect, screen_info, vertices.len() as u32);
            vertices.extend_from_slice(&new_vertices);
            indices.extend_from_slice(&new_indices);
        }

        // TODO: add other primatives -> figure out text rendering

        // make sure there is enough capcity on the gpu
        self.resize_vertex_buffer_if_required(device, vertices.len());
        self.resize_index_buffer_if_required(device, indices.len());

        // write data to the queue
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));

        // set data to be rendered
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);

        // handle text rendering with glyphon
        self.glyphon_renderer
            .render_text(device, queue, render_pass, screen_info, render_list);
    }

    fn resize_vertex_buffer_if_required(
        &mut self,
        device: &wgpu::Device,
        num_requested_vertices: usize,
    ) {
        if num_requested_vertices <= self.vertex_buffer_capacity {
            return;
        }
        let new_size = num_requested_vertices.next_power_of_two(); // we should grow exponentially in this case to avoid more resizes in the future
        self.vertex_buffer = get_vertex_buffer(device, Vertex::capacity_to_bytes(new_size));
        self.vertex_buffer_capacity = new_size;
    }

    fn resize_index_buffer_if_required(
        &mut self,
        device: &wgpu::Device,
        num_requested_indices: usize,
    ) {
        if num_requested_indices <= self.index_buffer_capacity {
            return;
        }
        let new_size = num_requested_indices.next_power_of_two();
        self.index_buffer = get_index_buffer(
            device,
            (new_size * std::mem::size_of::<u32>()) as wgpu::BufferAddress, // u32 for size of index
        );
        self.index_buffer_capacity = new_size;
    }
}
