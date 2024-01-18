use app_surface::AppSurface;
use wgpu::util::DeviceExt;

use crate::{compute::ComputeInstance, model};

/// The `Instance` struct represents an instance in 3D space with position and rotation.
///
/// Properties:
///
/// * `position`: The `position` property is a 3D vector that represents the position of an instance in
/// 3D space. It is typically used to store the x, y, and z coordinates of the instance's position.
/// * `rotation`: The `rotation` property is of type `glam::Quat`. It represents the rotation of an
/// instance in 3D space. A quaternion is a mathematical representation of a rotation that avoids the
/// problems of gimbal lock and provides smooth interpolation between rotations.
pub struct Instance {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
}

/// The `InstanceRaw` type represents a raw instance with model and normal matrices in Rust.
///
/// Properties:
///
/// * `model`: The `model` property is a 4x4 matrix of `f32` values. It represents the transformation
/// matrix that is used to position, rotate, and scale an object in 3D space. This matrix is typically
/// used to transform the vertices of a 3D model from model space
/// * `normal`: The `normal` property is a 3x3 matrix of `f32` values. It represents the normal matrix,
/// which is used to transform normal vectors in a 3D space. Normal vectors are used in lighting
/// calculations to determine how light interacts with a surface.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[allow(dead_code)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],  // model matrix
    normal: [[f32; 3]; 3], // normal matrix
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (glam::Mat4::from_translation(self.position)
                * glam::Mat4::from_quat(self.rotation))
            .to_cols_array_2d(),
            normal: glam::Mat3::from_mat4(glam::Mat4::from_quat(self.rotation)).to_cols_array_2d(),
        }
    }
}

impl ComputeInstance {
    pub fn to_render_instance_raw(&self) -> InstanceRaw {
        let model = glam::Mat4::from_translation(self.position).to_cols_array_2d();
        let normal = glam::Mat3::from_rotation_z(0.0).to_cols_array_2d();
        InstanceRaw { model, normal }
    }
}

impl model::Vertex for InstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We don't have to do this in code though.
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct InstanceState {
    pub instances_number: usize,
    #[allow(dead_code)]
    pub instance_buffer: wgpu::Buffer,
}

impl InstanceState {
    // 在 [-range, range] 范围内随机生成 instances_number 个 Instance
    pub fn new(app: &AppSurface, compute_instance: &[ComputeInstance]) -> Self {
        let instances_data = compute_instance
            .iter()
            .map(ComputeInstance::to_render_instance_raw)
            .collect::<Vec<_>>();
        let instance_buffer = app
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instances_data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        Self {
            instance_buffer,
            instances_number: instances_data.len(),
        }
    }

    pub fn update(&mut self, app: &AppSurface, compute_instance: &[ComputeInstance]) {
        self.instances_number = compute_instance.len();
        let instances_data = compute_instance
            .iter()
            .map(ComputeInstance::to_render_instance_raw)
            .collect::<Vec<_>>();
        // Update the instance buffer
        app.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instances_data),
        );
    }
}
