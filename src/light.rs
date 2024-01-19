use std::f32::consts;

use app_surface::AppSurface;
use wgpu::util::DeviceExt;

/// `LightUniform` 结构体表示用于在 Rust 程序中存储光信息的统一缓冲区对象。
///
/// Properties:
///
/// * `position`: 表示光源位置的 3D 矢量。
/// * `_padding`: `_padding` 字段用于确保 `color` 字段从内存中的 16 字节（4 个浮点数）边界开始。这是必要的，因为某些图形 API
/// 需要将统一数据与某些内存边界对齐以实现高效访问。 `_padding` 字段不用于任何
/// * `color`: “color”属性是一个由“f32”值组成的 3 元素数组，表示灯光的 RGB 颜色分量。每个分量的范围从 0.0 到 1.0，其中 0.0 表示无强度，1.0 表示全强度。
/// * `_padding2`: `_padding2` 字段用于填充，以确保 `color` 字段在内存中正确对齐。着色器中的 Uniform 通常要求元素之间有 16 字节（4
/// 个浮点）间距，因此添加填充字段以确保正确对齐。
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    color: [f32; 3],
    _padding2: u32,
}

/// `LightState` 结构表示 Rust 程序中光源的状态。
///
/// Properties:
///
/// * `light_uniform`: “light_uniform”属性是“LightUniform”类型的结构体或对象。它可能包含与光源属性相关的数据，例如其位置、颜色、强度等。
/// * `light_buffer`: “light_buffer”属性是存储灯光数据的缓冲区。它的类型为`wgpu::Buffer`，是wgpu库提供的缓冲区对象。缓冲区用于在 GPU
/// 上存储和操作数据。在这种情况下，`light_buffer
/// * `light_bind_group_layout`: “light_bind_group_layout”是一个布局，描述了将绑定到着色器的资源的绑定槽和类型。它定义着色器将使用的资源的结构和组织。
/// * `light_bind_group`:
/// “light_bind_group”是一个绑定组，表示可以绑定在一起以在着色器中使用的资源集合。它用于将“light_buffer”和其他资源绑定到着色器管道。
pub struct LightState {
    pub light_uniform: LightUniform,
    pub light_buffer: wgpu::Buffer,
    pub light_bind_group_layout: wgpu::BindGroupLayout,
    pub light_bind_group: wgpu::BindGroup,
}

impl LightState {
    pub fn new(app: &AppSurface) -> Self {
        let light_uniform = LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };

        let light_buffer = app
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Light Vertex Buffer"),
                contents: bytemuck::cast_slice(&[light_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let light_bind_group_layout =
            app.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: None,
                });

        let light_bind_group = app.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        Self {
            light_uniform,
            light_buffer,
            light_bind_group_layout,
            light_bind_group,
        }
    }
    pub fn update(&mut self, app: &AppSurface) {
        let old_position = glam::Vec3::from_array(self.light_uniform.position);
        self.light_uniform.position =
            (glam::Quat::from_axis_angle(glam::Vec3::Y, consts::PI / 180.) * old_position).into();
        app.queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[self.light_uniform]),
        );
    }
}
