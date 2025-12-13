// SPDX-License-Identifier: MPL-2.0
//! GPU-accelerated video frame rendering using custom wgpu shader.
//!
//! This widget provides flicker-free video playback by maintaining a persistent
//! GPU texture that is updated in-place, avoiding the texture recreation overhead
//! that occurs with Iced's standard Image widget.
//!
//! # Architecture
//!
//! `VideoShader` maintains a single GPU texture and updates it using
//! `queue.write_texture()`. This avoids the texture churn that would occur
//! if creating a new `image::Handle` for each frame (since `Handle::from_rgba()`
//! generates a unique ID per call, causing GPU texture recreation).

use crate::media::frame_export::ExportableFrame;
use iced::widget::shader::{self, Viewport};
use iced::{mouse, Element, Length, Rectangle};
use std::sync::Arc;
use wgpu;

/// Video frame data ready for GPU upload.
#[derive(Debug, Clone)]
pub struct FrameData {
    /// RGBA pixel data (width * height * 4 bytes)
    pub rgba: Arc<Vec<u8>>,
    /// Frame width in pixels
    pub width: u32,
    /// Frame height in pixels
    pub height: u32,
}

/// A GPU-accelerated video frame renderer using custom wgpu shaders.
///
/// This widget maintains a persistent GPU texture that is updated in-place
/// when new video frames arrive, eliminating the flickering caused by
/// texture recreation in the standard Image widget.
#[derive(Debug)]
pub struct VideoShader<Message> {
    /// Current frame data (if any)
    frame: Option<FrameData>,
    /// Display width (scaled)
    display_width: f32,
    /// Display height (scaled)
    display_height: f32,
    /// Zoom factor (1.0 = 100%)
    zoom: f32,
    /// Phantom data for message type
    _phantom: std::marker::PhantomData<Message>,
}

impl<Message> Default for VideoShader<Message> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Message> VideoShader<Message> {
    /// Creates a new VideoShader with no frame.
    pub fn new() -> Self {
        Self {
            frame: None,
            display_width: 0.0,
            display_height: 0.0,
            zoom: 1.0,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Sets a new video frame from RGBA pixel data.
    pub fn set_frame(&mut self, rgba_data: Arc<Vec<u8>>, width: u32, height: u32) {
        self.frame = Some(FrameData {
            rgba: rgba_data,
            width,
            height,
        });
        self.display_width = width as f32 * self.zoom;
        self.display_height = height as f32 * self.zoom;
    }

    /// Clears the current frame.
    pub fn clear_frame(&mut self) {
        self.frame = None;
        self.display_width = 0.0;
        self.display_height = 0.0;
    }

    /// Clears the current frame (alias for `clear_frame` for API compatibility).
    pub fn clear(&mut self) {
        self.clear_frame();
    }

    /// Returns true if a frame is loaded.
    pub fn has_frame(&self) -> bool {
        self.frame.is_some()
    }

    /// Sets the zoom factor.
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom;
        if let Some(ref frame) = self.frame {
            self.display_width = frame.width as f32 * zoom;
            self.display_height = frame.height as f32 * zoom;
        }
    }

    /// Sets the scale factor (alias for `set_zoom` for API compatibility).
    pub fn set_scale(&mut self, scale: f32) {
        self.set_zoom(scale);
    }

    /// Returns the current zoom factor.
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// Returns an exportable frame if one is available.
    ///
    /// This can be used to save the current frame to a file.
    pub fn exportable_frame(&self) -> Option<ExportableFrame> {
        self.frame
            .as_ref()
            .map(|f| ExportableFrame::new((*f.rgba).clone(), f.width, f.height))
    }

    /// Returns the current frame data if available.
    pub fn frame(&self) -> Option<&FrameData> {
        self.frame.as_ref()
    }

    /// Returns the raw RGBA data for frame export.
    pub fn raw_rgba_data(&self) -> Option<&Arc<Vec<u8>>> {
        self.frame.as_ref().map(|f| &f.rgba)
    }

    /// Returns the frame dimensions.
    pub fn dimensions(&self) -> Option<(u32, u32)> {
        self.frame.as_ref().map(|f| (f.width, f.height))
    }

    /// Returns the scaled display width.
    pub fn scaled_width(&self) -> f32 {
        self.display_width
    }

    /// Returns the scaled display height.
    pub fn scaled_height(&self) -> f32 {
        self.display_height
    }

    /// Creates an Element for rendering this video frame.
    pub fn view(&self) -> Element<'_, Message>
    where
        Message: 'static,
    {
        if let Some(ref frame) = self.frame {
            let program = VideoFrameProgram {
                frame: frame.clone(),
            };

            // The widget size is handled by Iced's layout system.
            // The shader's viewport will be set to the visible clip_bounds,
            // and the fullscreen quad will fill that area.
            shader::Shader::new(program)
                .width(Length::Fixed(self.display_width.max(1.0)))
                .height(Length::Fixed(self.display_height.max(1.0)))
                .into()
        } else {
            iced::widget::Space::new()
                .width(Length::Fixed(1.0))
                .height(Length::Fixed(1.0))
                .into()
        }
    }
}

/// The shader program for rendering a video frame.
#[derive(Debug, Clone)]
struct VideoFrameProgram {
    frame: FrameData,
}

impl<Message> shader::Program<Message> for VideoFrameProgram {
    type State = ();
    type Primitive = VideoFramePrimitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        _bounds: Rectangle,
    ) -> Self::Primitive {
        VideoFramePrimitive {
            frame: self.frame.clone(),
        }
    }
}

/// The rendering primitive for a video frame.
#[derive(Debug, Clone)]
pub struct VideoFramePrimitive {
    frame: FrameData,
}

impl shader::Primitive for VideoFramePrimitive {
    type Pipeline = VideoPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        // Store the widget's full physical bounds for use in render().
        // This is critical for correct rendering when zoomed/scrolled:
        // - bounds = the full widget size (frame * zoom) in logical coords
        // - We convert to physical pixels and store for render()
        pipeline.store_physical_bounds(bounds, viewport);

        // Update texture with new frame data
        pipeline.update_frame(device, queue, &self.frame);
    }

    fn render(
        &self,
        pipeline: &Self::Pipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        pipeline.render(encoder, target, clip_bounds);
    }
}

// Note: No uniform buffer needed - we use viewport-based positioning instead.
// The viewport transformation handles quad positioning automatically when
// we render a fullscreen quad (-1 to 1 in NDC).

/// The wgpu pipeline for rendering video frames.
pub struct VideoPipeline {
    pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    // Current texture and bind group (recreated when frame size changes)
    texture: Option<wgpu::Texture>,
    texture_bind_group: Option<wgpu::BindGroup>,
    current_size: (u32, u32),
    // Store the full widget bounds (in physical pixels) from prepare() for use in render()
    // This is needed because render() only receives clip_bounds (the visible portion)
    widget_physical_bounds: Rectangle<f32>,
}

impl shader::Pipeline for VideoPipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Video Frame Shader"),
            source: wgpu::ShaderSource::Wgsl(VIDEO_SHADER.into()),
        });

        // Create sampler for texture
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Video Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create texture bind group layout (group 0)
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Video Texture Bind Group Layout"),
                entries: &[
                    // Texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // Create pipeline layout - only texture bind group, no uniforms needed
        // We use viewport-based positioning which handles quad placement automatically
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Video Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Video Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            texture_bind_group_layout,
            sampler,
            texture: None,
            texture_bind_group: None,
            current_size: (0, 0),
            widget_physical_bounds: Rectangle::default(),
        }
    }
}

impl VideoPipeline {
    /// Store the widget's physical bounds for use in render().
    /// This converts logical bounds to physical pixels using the viewport's scale factor.
    fn store_physical_bounds(&mut self, bounds: &Rectangle, viewport: &Viewport) {
        let scale = viewport.scale_factor();
        self.widget_physical_bounds = Rectangle {
            x: bounds.x * scale,
            y: bounds.y * scale,
            width: bounds.width * scale,
            height: bounds.height * scale,
        };
    }

    fn update_frame(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, frame: &FrameData) {
        let new_size = (frame.width, frame.height);

        // Recreate texture if size changed or doesn't exist
        if self.texture.is_none() || self.current_size != new_size {
            self.create_texture(device, frame.width, frame.height);
            self.current_size = new_size;
        }

        // Update texture data in place
        if let Some(ref texture) = self.texture {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &frame.rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(frame.width * 4),
                    rows_per_image: Some(frame.height),
                },
                wgpu::Extent3d {
                    width: frame.width,
                    height: frame.height,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    fn create_texture(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Video Frame Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Use Rgba8Unorm (not Srgb) because video frames from ffmpeg are already
            // gamma-corrected. Using Rgba8UnormSrgb would apply double gamma correction,
            // making the video appear darker.
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Video Texture Bind Group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        self.texture = Some(texture);
        self.texture_bind_group = Some(bind_group);
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        let Some(ref texture_bind_group) = self.texture_bind_group else {
            return;
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Video Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, texture_bind_group, &[]);

        // Set viewport to the FULL widget bounds (stored from prepare()).
        // This ensures the video maintains correct aspect ratio when zoomed.
        // The fullscreen quad (-1 to 1 in NDC) will fill this viewport,
        // and the scissor rect will clip to the visible portion.
        let wb = &self.widget_physical_bounds;
        render_pass.set_viewport(wb.x, wb.y, wb.width, wb.height, 0.0, 1.0);

        // Set scissor rect to clip_bounds to only render the visible portion.
        // This is essential for correct clipping inside scrollables.
        render_pass.set_scissor_rect(
            clip_bounds.x,
            clip_bounds.y,
            clip_bounds.width,
            clip_bounds.height,
        );

        render_pass.draw(0..4, 0..1);
    }
}

/// WGSL shader for video frame rendering.
///
/// This shader renders a fullscreen quad that fills the entire viewport.
/// The viewport is set to the widget's clip_bounds in the render() method,
/// so the quad automatically fills the correct area on screen.
const VIDEO_SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Generate a fullscreen quad using triangle strip
    // Vertex order: top-left(0), top-right(1), bottom-left(2), bottom-right(3)
    // This fills the entire NDC space (-1 to 1), which maps to the viewport
    let x = f32(vertex_index & 1u);        // 0, 1, 0, 1
    let y = f32(vertex_index >> 1u);       // 0, 0, 1, 1

    // Map to NDC: x from -1 to 1, y from 1 to -1 (flipped for screen coords)
    let pos_x = x * 2.0 - 1.0;             // -1, 1, -1, 1
    let pos_y = 1.0 - y * 2.0;             // 1, 1, -1, -1

    var output: VertexOutput;
    output.position = vec4<f32>(pos_x, pos_y, 0.0, 1.0);
    output.tex_coord = vec2<f32>(x, y);
    return output;
}

@group(0) @binding(0)
var video_texture: texture_2d<f32>;
@group(0) @binding(1)
var video_sampler: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(video_texture, video_sampler, input.tex_coord);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn video_shader_default_has_no_frame() {
        let shader: VideoShader<()> = VideoShader::new();
        assert!(!shader.has_frame());
    }

    #[test]
    fn video_shader_set_frame_updates_dimensions() {
        let mut shader: VideoShader<()> = VideoShader::new();
        let data = Arc::new(vec![0u8; 100 * 50 * 4]);
        shader.set_frame(data, 100, 50);

        assert!(shader.has_frame());
        assert_eq!(shader.dimensions(), Some((100, 50)));
        assert_eq!(shader.scaled_width(), 100.0);
        assert_eq!(shader.scaled_height(), 50.0);
    }

    #[test]
    fn video_shader_zoom_scales_display() {
        let mut shader: VideoShader<()> = VideoShader::new();
        let data = Arc::new(vec![0u8; 100 * 50 * 4]);
        shader.set_frame(data, 100, 50);
        shader.set_zoom(2.0);

        assert_eq!(shader.scaled_width(), 200.0);
        assert_eq!(shader.scaled_height(), 100.0);
    }

    #[test]
    fn video_shader_clear_frame_removes_data() {
        let mut shader: VideoShader<()> = VideoShader::new();
        let data = Arc::new(vec![0u8; 100 * 50 * 4]);
        shader.set_frame(data, 100, 50);
        shader.clear_frame();

        assert!(!shader.has_frame());
        assert_eq!(shader.scaled_width(), 0.0);
    }
}
