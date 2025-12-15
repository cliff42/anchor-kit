use std::collections::HashMap;

use anchor_kit_core::{
    primitives::rectangle::Rectangle,
    render::RenderList,
    style::{FontFamily, FontStyle, FontWeight},
};
use glyphon::{
    Attrs, Cache, FontSystem, Metrics, Shaping, SwashCache, TextArea, TextAtlas, TextBounds,
    TextRenderer, Viewport,
};
use image::GenericImageView;
use uuid::Uuid;
use wgpu::include_wgsl;

pub struct ScreenInfo {
    pub size_px: [u32; 2], // w, h
    pub scale_factor: f32, // used by glyphon to scale text
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::NoUninit)] // TODO: do we need to go back to bytemuck POD/ zeroable here instead?
struct Vertex {
    position: [f32; 2],         // x, y (normalized)
    local_uv: [f32; 2],         // uv in local units (inside the object)
    background_color: [f32; 4], // r, g, b, a
    border_radius: [f32; 4],    // top-left, top-right, bottom-right, bottom-left (clockwise)
    border_width: f32,
    border_color: [f32; 4], // r, g, b, a
    scale: [f32; 2],        // scale x,y to w,h
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
        0 => Float32x2, // location 0 is normalized position
        1 => Float32x2, // location 1 is uv in local units within the object
        2 => Float32x4, // location 2 is colour
        3 => Float32x4, // location 3 is border radius (also in local units)
        4 => Float32, // location 4 is border width (also in local units)
        5 => Float32x4, // location 5 is boder colour
        6 => Float32x2, // location 6 is the scale
    ];

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
    let scale_axis = w.min(h) as f32;
    let scale = if scale_axis <= 0.0 {
        [1.0, 1.0]
    } else {
        [(w as f32) / scale_axis, (h as f32) / scale_axis]
    };
    let [screen_w, screen_h] = screen_info.size_px;

    // normalize pixel values
    let x0 = x as f32 / screen_w as f32;
    let x1 = (x + w) as f32 / screen_w as f32;
    let y0 = y as f32 / screen_h as f32;
    let y1 = (y + h) as f32 / screen_h as f32;

    let background_color = rect.style.background_color.to_rgba_f32();
    let border_color = rect.style.border_color.to_rgba_f32();

    // convert radius to local units
    let mut local_radius = rect.style.border_radius;
    for r in local_radius.iter_mut() {
        *r = (*r / w.min(h) as f32).min(0.5) // we don't want the radius exceeding 0.5 to avoid impossible rounded corners
    }
    let local_border_width = rect.style.border_width / w.min(h) as f32;

    // for the vertices the local uv values are just the corners
    let v0 = Vertex {
        position: [x0, y0],
        local_uv: [0.0, 0.0],
        background_color,
        border_radius: local_radius,
        border_width: local_border_width,
        border_color,
        scale,
    };
    let v1 = Vertex {
        position: [x1, y0],
        local_uv: [1.0, 0.0],
        background_color,
        border_radius: local_radius,
        border_width: local_border_width,
        border_color,
        scale,
    };
    let v2 = Vertex {
        position: [x1, y1],
        local_uv: [1.0, 1.0],
        background_color,
        border_radius: local_radius,
        border_width: local_border_width,
        border_color,
        scale,
    };
    let v3 = Vertex {
        position: [x0, y1],
        local_uv: [0.0, 1.0],
        background_color,
        border_radius: local_radius,
        border_width: local_border_width,
        border_color,
        scale,
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
            let text_style = &text_item.text_style;

            // TODO: metrics should be set by text style passed in by user
            let mut text_buffer = glyphon::Buffer::new(
                &mut self.font_system,
                Metrics::new(text_style.font_size, text_style.line_height),
            );

            text_buffer.set_size(
                &mut self.font_system,
                Some(physical_width),
                Some(physical_height),
            );

            let text_color = glyphon::Color::rgba(
                text_item.text_style.text_color.r,
                text_item.text_style.text_color.g,
                text_item.text_style.text_color.b,
                text_item.text_style.text_color.a,
            );

            let text_attrs = Attrs::new()
                .family(Self::anchor_kit_font_family_to_glyphon(
                    &text_style.font_family,
                ))
                .style(Self::anchor_kit_font_style_to_glyphon(
                    &text_style.font_style,
                ))
                .weight(Self::anchor_kit_font_weight_to_glyphon(
                    &text_style.font_weight,
                ))
                .color(text_color);

            text_buffer.set_text(
                &mut self.font_system,
                &text_item.text,
                &text_attrs,
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
                text_item.text_style.text_color.r,
                text_item.text_style.text_color.g,
                text_item.text_style.text_color.b,
                text_item.text_style.text_color.a,
            );

            text_areas.push(TextArea {
                buffer: &text_buffer,
                left: x as f32,
                top: y as f32,
                scale: 1.0, // ignore screen scale factor (TODO: investigate if we want to include this later)
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

    fn anchor_kit_font_family_to_glyphon(font_family: &FontFamily) -> glyphon::Family<'_> {
        match font_family {
            FontFamily::Name(name) => glyphon::Family::Name(name),
            FontFamily::Serif => glyphon::Family::Serif,
            FontFamily::SansSerif => glyphon::Family::SansSerif,
            FontFamily::Cursive => glyphon::Family::Cursive,
            FontFamily::Fantasy => glyphon::Family::Fantasy,
            FontFamily::Monospace => glyphon::Family::Monospace,
        }
    }

    fn anchor_kit_font_weight_to_glyphon(font_weight: &FontWeight) -> glyphon::Weight {
        match font_weight {
            FontWeight::Thin => glyphon::Weight::THIN,
            FontWeight::ExtraLight => glyphon::Weight::EXTRA_LIGHT,
            FontWeight::Light => glyphon::Weight::LIGHT,
            FontWeight::Normal => glyphon::Weight::NORMAL,
            FontWeight::Medium => glyphon::Weight::MEDIUM,
            FontWeight::SemiBold => glyphon::Weight::SEMIBOLD,
            FontWeight::Bold => glyphon::Weight::BOLD,
            FontWeight::ExtraBold => glyphon::Weight::EXTRA_BOLD,
            FontWeight::Black => glyphon::Weight::BLACK,
        }
    }

    fn anchor_kit_font_style_to_glyphon(font_style: &FontStyle) -> glyphon::Style {
        match font_style {
            FontStyle::Normal => glyphon::Style::Normal,
            FontStyle::Italic => glyphon::Style::Italic,
            FontStyle::Oblique => glyphon::Style::Oblique,
        }
    }
}

pub struct Renderer {
    main_pipeline: wgpu::RenderPipeline,
    image_pipeline: wgpu::RenderPipeline, // we need a new pipeline for iamges because we have to pass bind groups to the fragment shader
    vertex_buffer: wgpu::Buffer,
    vertex_buffer_capacity: usize,
    index_buffer: wgpu::Buffer,
    index_buffer_capacity: usize,
    glyphon_renderer: GlyphonRenderer,
    bind_groups: HashMap<Uuid, wgpu::BindGroup>, // we want to store a map of ids to texture bind groups so we don't have to regenerate them each frame
    texture_bind_group_layout: wgpu::BindGroupLayout, // we only need one bind group layout for all textures
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

        let main_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("anchor-kit main layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let main_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("anchor-kit render pipeline"),
            layout: Some(&main_pipeline_layout),
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

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("anchor-kit bindgroup layout"),
            });

        let image_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("anchor-kit image pipeline layout"),
                bind_group_layouts: &[&texture_bind_group_layout], // image pipeline layout needs the texture bind group layout
                push_constant_ranges: &[],
            });

        // create seperate image pipeline for rendering images (using textures)
        let image_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("anchor-kit render pipeline"),
            layout: Some(&image_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()], // get the buffer layout description from the vertex impl
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_image"), // use the image fragment shader
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
            main_pipeline,
            image_pipeline,
            vertex_buffer,
            vertex_buffer_capacity: initial_vertex_buffer_capacity as usize,
            index_buffer,
            index_buffer_capacity: initial_index_buffer_capacity as usize,
            glyphon_renderer: GlyphonRenderer::new(device, queue, texture_format),
            bind_groups: HashMap::new(),
            texture_bind_group_layout,
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
        // main pipeline rendering
        let mut vertices: Vec<Vertex> = vec![];
        let mut indices: Vec<u32> = vec![];

        // convert all primatives to vertices
        let main_pipeline_index_offset = indices.len();
        for rect in &render_list.rectangles {
            // offset will increment as new vertices are added
            let (new_vertices, new_indices) =
                get_vertices_and_indices_for_rectangle(rect, screen_info, vertices.len() as u32);
            vertices.extend_from_slice(&new_vertices);
            indices.extend_from_slice(&new_indices);
        }
        let main_pipeline_index_count = indices.len();

        // we will keep track of image draws seperatly so that we can use the correct bind gorups later for the texture rendering
        struct ImageDraw {
            texture_id: Uuid,
            index_offset: usize, // where this image starts in the list of shared indices
            index_count: usize,
        }
        let mut image_draws: Vec<ImageDraw> = vec![];

        for image in &render_list.images {
            let image_index_offset = indices.len();

            let (new_vertices, new_indices) = get_vertices_and_indices_for_rectangle(
                &image.rectangle,
                screen_info,
                vertices.len() as u32,
            );
            vertices.extend_from_slice(&new_vertices);
            indices.extend_from_slice(&new_indices);

            image_draws.push(ImageDraw {
                texture_id: image.texture_id,
                index_offset: image_index_offset,
                index_count: new_indices.len(),
            })
        }

        // make sure there is enough capcity on the gpu
        self.resize_vertex_buffer_if_required(device, vertices.len());
        self.resize_index_buffer_if_required(device, indices.len());

        // write data to the queue
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        // set data to be rendered (for main pipeline (for rectangles))
        render_pass.set_pipeline(&self.main_pipeline);
        render_pass.draw_indexed(
            main_pipeline_index_offset as u32..main_pipeline_index_count as u32, // use the main pipeline index count/ offset
            0,
            0..1,
        );

        // draw the images using the image pipeline (individual draws for each since they could have different textures)
        render_pass.set_pipeline(&self.image_pipeline);
        for image_draw in image_draws.iter() {
            if let Some(bind_group) = self.bind_groups.get(&image_draw.texture_id) {
                render_pass.set_bind_group(0, bind_group, &[]);
                render_pass.draw_indexed(
                    image_draw.index_offset as u32
                        ..(image_draw.index_offset + image_draw.index_count) as u32, // use the specific image index count/ offset
                    0,
                    0..1,
                );
            }
        }

        // handle text rendering with glyphon
        self.glyphon_renderer
            .render_text(device, queue, render_pass, screen_info, render_list);
    }

    // image/ texture rendering inspired by: https://sotrh.github.io/learn-wgpu/beginner/tutorial5-textures/#the-bindgroup
    pub fn get_image_id_from_bytes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        diffuse_bytes: &[u8],
    ) -> Uuid {
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();
        let dimensions = diffuse_image.dimensions();

        // create the texture
        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1, // we keep depth as 1 for 2d images
        };

        let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("anchor-kit diffuse texture"),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &diffuse_rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let diffuse_texture_view =
            diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout, // the layout is created in the new function (single layout for all texture bind groups)
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                },
            ],
            label: Some("anchor-kit diffuse bind group"),
        });

        // generate the unique texture id so we can access it later without regenerating
        let id = Uuid::new_v4();
        self.bind_groups.insert(id, diffuse_bind_group);
        id // return the id so the user can use it to render images in the generate frame pass
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
