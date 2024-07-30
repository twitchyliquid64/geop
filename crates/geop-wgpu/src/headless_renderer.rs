use geop_rasterize::{
    edge::{rasterize_edges_into_line_list, rasterize_edges_into_vertex_list},
    edge_buffer::EdgeBuffer,
    triangle_buffer::TriangleBuffer,
    vertex_buffer::{RenderVertex, VertexBuffer},
    volume::{
        rasterize_volume_into_face_list, rasterize_volume_into_line_list,
        rasterize_volume_into_vertex_list,
    },
};
use geop_topology::topology::scene::Scene;
use winit::dpi::PhysicalSize;

use crate::pipeline_manager::PipelineManager;

pub struct HeadlessRenderer {
    pipeline_manager: PipelineManager,
    queue: wgpu::Queue,
    device: wgpu::Device,
    texture_view: wgpu::TextureView,
    output_buffer: wgpu::Buffer,
    texture: wgpu::Texture,
    texture_size: u32,
    copy_size: wgpu::Extent3d,
}

impl HeadlessRenderer {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&Default::default(), None)
            .await
            .unwrap();
        let texture_size = 1024u32;

        let texture_format = wgpu::TextureFormat::Rgba8UnormSrgb;

        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: texture_size,
                height: texture_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: texture_format,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[texture_format],
        };
        let texture = device.create_texture(&texture_desc);
        let texture_view = texture.create_view(&Default::default());

        let pipeline_manager = PipelineManager::new(
            &device,
            PhysicalSize::new(texture_size, texture_size),
            texture_format,
        )
        .await;

        // we need to store this for later
        let u32_size = std::mem::size_of::<u32>() as u32;

        let output_buffer_size = (u32_size * texture_size * texture_size) as wgpu::BufferAddress;
        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST
        // this tells wpgu that we want to read this buffer from the cpu
        | wgpu::BufferUsages::MAP_READ,
            label: None,
            mapped_at_creation: false,
        };
        let output_buffer = device.create_buffer(&output_buffer_desc);

        // We need to scope the mapping variables so that we can
        // unmap the buffer

        HeadlessRenderer {
            pipeline_manager,
            queue,
            device,
            texture_view,
            output_buffer,
            texture,
            texture_size,
            copy_size: texture_desc.size,
        }
    }

    pub async fn render_to_file(
        &mut self,
        scene: &Scene,
        dark_mode: bool,
        file_path: &std::path::Path,
    ) {
        let background_color = if dark_mode {
            [0.02, 0.02, 0.02, 1.0]
        } else {
            [1.0, 1.0, 1.0, 1.0]
        };
        let face_color = if dark_mode {
            [0.2, 0.2, 0.2, 1.0]
        } else {
            [0.6, 0.6, 0.6, 1.0]
        };
        let edge_color = if dark_mode {
            [0.7, 0.7, 0.7, 1.0]
        } else {
            [0.2, 0.2, 0.2, 1.0]
        };
        let point_color = if dark_mode {
            [0.8, 0.8, 0.8, 1.0]
        } else {
            [0.1, 0.1, 0.1, 1.0]
        };

        let u32_size = std::mem::size_of::<u32>() as u32;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let render_pass_desc = wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: background_color[0],
                            g: background_color[1],
                            b: background_color[2],
                            a: background_color[3],
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            };
            let mut vertex_buffer = VertexBuffer::empty();
            let mut edge_buffer = EdgeBuffer::empty();
            let mut triangle_buffer = TriangleBuffer::empty();

            for volume in scene.volumes.iter() {
                vertex_buffer.join(&rasterize_volume_into_vertex_list(volume, point_color));
                edge_buffer.join(&rasterize_volume_into_line_list(volume, edge_color));
                triangle_buffer.join(&rasterize_volume_into_face_list(volume, face_color));
            }

            vertex_buffer.join(&rasterize_edges_into_vertex_list(&scene.edges, point_color));
            edge_buffer.join(&rasterize_edges_into_line_list(&scene.edges, edge_color));

            vertex_buffer.join(&VertexBuffer::new(
                scene
                    .points
                    .iter()
                    .map(|p| RenderVertex::new(p.clone(), point_color))
                    .collect(),
            ));

            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
            self.pipeline_manager
                .update_edges(&self.queue, &edge_buffer);
            self.pipeline_manager
                .update_triangles(&self.queue, &triangle_buffer);
            self.pipeline_manager
                .update_vertices(&self.queue, &vertex_buffer);
            self.pipeline_manager.run_pipelines(&mut render_pass);
        }

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(u32_size * self.texture_size),
                    rows_per_image: Some(self.texture_size),
                },
            },
            self.copy_size,
        );
        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = self.output_buffer.slice(..);

        // NOTE: We have to create the mapping THEN device.poll() before await
        // the future. Otherwise the application will freeze.
        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.receive().await.unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();

        use image::{ImageBuffer, Rgba};
        let buffer =
            ImageBuffer::<Rgba<u8>, _>::from_raw(self.texture_size, self.texture_size, data)
                .unwrap();
        buffer.save(file_path).unwrap();
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use geop_topology::primitive_objects::volumes::cube::primitive_cube;
    use rstest::{fixture, rstest};

    #[fixture]
    pub async fn renderer() -> Box<HeadlessRenderer> {
        Box::new(HeadlessRenderer::new().await)
    }
}
