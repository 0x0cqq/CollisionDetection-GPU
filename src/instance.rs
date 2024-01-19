use app_surface::AppSurface;
use wgpu::util::DeviceExt;

use crate::{compute::ComputeInstance, model};

/// `InstanceRaw` 类型表示 Rust 中具有模型和普通矩阵的原始实例。
///
/// Properties:
///
/// * `model`: 表示实例模型转换的 4x4 矩阵。该矩阵用于在 3D 空间中定位、旋转和缩放实例。矩阵的每个元素都是一个 32 位浮点数 (f32)。
/// * `normal`: “normal”属性是“f32”值的 3x3 矩阵。它表示法线矩阵，用于在 3D 空间中变换法线向量。法线向量用于照明计算，以确定光如何与表面相互作用。
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[allow(dead_code)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],  // model matrix
    normal: [[f32; 3]; 3], // normal matrix
}

impl ComputeInstance {
    /// “to_render_instance_raw”函数返回一个“InstanceRaw”结构，其中包含用于渲染的模型和法线矩阵。
    ///
    /// Returns:
    ///
    /// `InstanceRaw` 结构的一个实例。
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

/// `InstanceState` 结构体表示 Rust 程序中实例的状态，包括实例的数量和用于存储实例数据的缓冲区。
///
/// Properties:
///
/// * `instances_number`: 表示实例数量的无符号整数。此属性用于跟踪实例状态中的实例数量。
/// * `instance_buffer`: `instance_buffer` 是 `wgpu::Buffer` 类型的属性。它是一个存储实例数据的缓冲区。
pub struct InstanceState {
    pub instances_number: usize,
    #[allow(dead_code)]
    pub instance_buffer: wgpu::Buffer,
}

impl InstanceState {
    /// 该函数在 Rust 中为给定的应用程序表面和计算实例创建一个新的实例缓冲区。
    ///
    /// Arguments:
    ///
    /// * `app`: “AppSurface”结构的实例，表示将在其中呈现实例的应用程序表面。
    /// * `compute_instance`: `ComputeInstance` 对象的切片。
    ///
    /// Returns:
    ///
    /// `Self` 结构的一个实例。
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

    /// “update”函数使用来自“compute_instance”向量的数据更新实例缓冲区。
    ///
    /// Arguments:
    ///
    /// * `app`: “AppSurface”结构的实例，表示将发生渲染的应用程序表面或窗口。
    /// * `compute_instance`: “compute_instance”是“ComputeInstance”对象的一部分。
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
