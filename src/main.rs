use std::iter;

use app_surface::{AppSurface, SurfaceFrame};
use winit::{event::*, window::WindowId};

mod framework;
mod light;
use framework::run;
mod camera;
mod compute;
mod instance;
mod model;
mod resources;
mod texture;
mod utils;

use model::{DrawLight, DrawModel, Vertex};

struct State {
    app: AppSurface,
    // pipelines
    render_pipeline: wgpu::RenderPipeline,
    light_render_pipeline: wgpu::RenderPipeline,
    obj_model: model::Model,
    depth_texture: texture::Texture,
    // camera related
    camera_state: camera::CameraState,
    // light related
    light_state: light::LightState,
    // Instances related
    instance_state: instance::InstanceState,
}

impl State {
    async fn new(app: AppSurface) -> Self {
        // Camera
        let camera_state = camera::CameraState::new(&app);
        // Light
        let light_state = light::LightState::new(&app);
        // Instances
        let instance_state = instance::InstanceState::new(&app);

        let texture_bind_group_layout =
            app.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        // normal map
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                });

        let depth_texture =
            texture::Texture::create_depth_texture(&app.device, &app.config, "depth_texture");

        let light_render_pipeline = {
            let layout = app
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Light Pipeline Layout"),
                    bind_group_layouts: &[
                        &camera_state.camera_bind_group_layout,
                        &light_state.light_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/light.wgsl").into()),
            };
            utils::create_render_pipeline(
                &app.device,
                &layout,
                app.config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc()],
                shader,
            )
        };

        let render_pipeline_layout =
            app.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &texture_bind_group_layout,
                        &camera_state.camera_bind_group_layout,
                        &light_state.light_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/draw.wgsl").into()),
            };
            utils::create_render_pipeline(
                &app.device,
                &render_pipeline_layout,
                app.config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), instance::InstanceRaw::desc()],
                shader,
            )
        };



        // 统一的用来画的模型（目前是一个正方体）
        let obj_model = resources::load_model(
            "sphere.obj",
            &app.device,
            &app.queue,
            &texture_bind_group_layout,
        )
        .await
        .unwrap();

        Self {
            app,
            render_pipeline,
            light_render_pipeline,
            obj_model,
            camera_state,
            light_state,
            instance_state,
            depth_texture,
        }
    }

    fn get_adapter_info(&self) -> wgpu::AdapterInfo {
        self.app.adapter.get_info()
    }

    fn current_window_id(&self) -> WindowId {
        self.app.view.id()
    }

    fn request_redraw(&mut self) {
        self.app.view.request_redraw();
    }

    /// The `resize` function resizes the application's projection, surface, and depth texture based on the
    /// new size provided.
    ///
    /// Arguments:
    ///
    /// * `new_size`: The `new_size` parameter is of type `winit::dpi::PhysicalSize<u32>`. It represents the
    /// new size of the window or surface that the code is resizing. It contains the width and height of the
    /// new size in pixels.
    fn resize(&mut self, new_size: &winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.camera_state.projection.resize(new_size.width, new_size.height);
            self.app.resize_surface();
            self.depth_texture = texture::Texture::create_depth_texture(
                &self.app.device,
                &self.app.config,
                "depth_texture",
            );
        }
    }

    /// This function handles various input events such as keyboard input, mouse wheel scrolling, and mouse
    /// button clicks.
    ///
    /// Arguments:
    ///
    /// * `event`: The `event` parameter is of type `WindowEvent`, which represents an event that occurred
    /// on the window.
    ///
    /// Returns:
    ///
    /// a boolean value.
    fn input(&mut self, event: &WindowEvent) -> bool {
        return self.camera_state.input(event);
    }

    /// This function updates the camera and light based on the controller and writes the updated data to
    /// buffers.
    ///
    /// Arguments:
    ///
    /// * `dt`: The `dt` parameter stands for "delta time" and represents the time elapsed since the last
    /// frame update. It is of type `std::time::Duration`, which is a struct that represents a span of time
    /// with nanosecond precision. In this code snippet, `dt` is used to update the camera and light based
    /// on the controller.
    fn update(&mut self, dt: std::time::Duration) {
        // Update the camera based on the controller
        self.camera_state.update(&self.app, dt);
        // Update the light position
        self.light_state.update(&self.app);
        // Update the instances
        self.instance_state.update(&self.app);

    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let (output, view) = self.app.get_current_frame_view(None);
        let mut encoder = self
            .app
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            render_pass.set_vertex_buffer(1, self.instance_state.instance_buffer.slice(..));
            render_pass.set_pipeline(&self.light_render_pipeline);
            render_pass.draw_light_model(
                &self.obj_model,
                &self.camera_state.camera_bind_group,
                &self.light_state.light_bind_group,
            );

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw_model_instanced(
                &self.obj_model,
                0..self.instance_state.instances.len() as u32,
                &self.camera_state.camera_bind_group,
                &self.light_state.light_bind_group,
            );
        }

        self.app.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    run(None);
}
