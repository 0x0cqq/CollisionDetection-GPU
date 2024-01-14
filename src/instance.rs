use app_surface::AppSurface;
use rand::Rng;
use wgpu::util::DeviceExt;

use crate::model;

const NUM_INSTANCES_PER_ROW: u32 = 1;


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
    // instance related
    pub instances: Vec<Instance>,
    #[allow(dead_code)]
    pub instance_buffer: wgpu::Buffer,
}

impl InstanceState {
    pub fn new(app: &AppSurface) -> Self {
        const SPACE_BETWEEN: f32 = 3.0;
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                    let position = glam::Vec3 { x, y: 0.0, z };

                    let rotation = if position.length().abs() <= std::f32::EPSILON {
                        glam::Quat::from_axis_angle(glam::Vec3::Z, 0.0)
                    } else {
                        glam::Quat::from_axis_angle(position.normalize(), std::f32::consts::FRAC_PI_4)
                    };

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances
            .iter()
            .map(Instance::to_raw)
            .collect::<Vec<_>>();
        let instance_buffer = app
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        Self {
            instances,
            instance_buffer,
        }
    }
    
    pub fn update(&mut self, app: &AppSurface) {
        let mut rng = rand::thread_rng();
        let instance = &mut self.instances[0];
        instance.position.x += rng.gen_range(-0.1..0.1);
        instance.position.z += rng.gen_range(-0.1..0.1);
        instance.rotation = if instance.position.length().abs() <= std::f32::EPSILON {
            glam::Quat::from_axis_angle(glam::Vec3::Z, 0.0)
        } else {
            glam::Quat::from_axis_angle(instance.position.normalize(), std::f32::consts::FRAC_PI_4)
        };

        // Update the instance buffer
        app.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(
                &self
                    .instances
                    .iter()
                    .map(|instance| instance.to_raw())
                    .collect::<Vec<_>>(),
            ),
        );
    }
}