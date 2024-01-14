use std::{sync::Arc, iter};
use rand::Rng;

use app_surface::AppSurface;
use wgpu::{ShaderModule, include_wgsl};

pub struct ComputeTestNode {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    workgroup_count: u32, // 一维的 workgroup 数量
    input_buffer: Arc<wgpu::Buffer>,
    output_buffer: Arc<wgpu::Buffer>,
}

pub const BUFFER_LEN: usize = 100;

// 测试一个简单的 compute shader
// 输入一个 u32 数组，输出两倍的 u32 数组
impl ComputeTestNode {
    pub fn new(app: &AppSurface, shader: ShaderModule, workgroup_count: u32) -> Self {
        let bind_group_layout =
            app.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Test Layout"),
                    entries: &[
                        // 一个只读的存储缓冲区，用于输入
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // 一个只写的存储缓冲区，用于输出
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });
        let pipeline_layout = app
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Test Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });
        let pipeline = app
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Test Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: "cs_main",
            });

        let input_buffer = Arc::new(app.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Input Buffer"),
            size: BUFFER_LEN as u64 * std::mem::size_of::<i32>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        let output_buffer = Arc::new(app.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: BUFFER_LEN as u64 * std::mem::size_of::<i32>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        }));

        let bind_group = app.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &input_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &output_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        Self {
            pipeline,
            bind_group,
            workgroup_count,
            input_buffer,
            output_buffer,
        }
    }
    pub fn dispatch<'a, 'b: 'a>(&'b self, cpass: &mut wgpu::ComputePass<'a>) {
        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &self.bind_group, &[]);
        cpass.dispatch_workgroups(self.workgroup_count, 1, 1);
    }

    pub fn write_input_buffer(&self, app: &AppSurface, data: &[i32]) {
        app.queue
            .write_buffer(&self.input_buffer, 0, bytemuck::cast_slice(&data));
    }
    pub fn read_output_buffer(&self) {
        let output_buffer = self.output_buffer.clone();

        output_buffer
            .clone()
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                result.expect("failed to map storage buffer");
                let binding = output_buffer.clone();
                let contents = binding.slice(..).get_mapped_range();
                let readback = contents
                    .chunks_exact(std::mem::size_of::<i32>())
                    .map(|bytes| i32::from_ne_bytes(bytes.try_into().unwrap()))
                    .collect::<Vec<_>>();
                println!("Output: {readback:?}");
            })
    }
}

pub fn do_compute(app: &AppSurface) {
    let compute_shader = app
        .device
        .create_shader_module(include_wgsl!("../shaders/compute.wgsl"));

    let computer_node =
        ComputeTestNode::new(&app, compute_shader, BUFFER_LEN as u32);

    let mut encoder = app
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Encoder"),
        });

    // generate input buffer
    let input_data: [i32; BUFFER_LEN] =
        core::array::from_fn(|_| rand::thread_rng().gen_range(0..123));

    computer_node.write_input_buffer(&app, &input_data);

    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute pass"),
            ..Default::default()
        });

        computer_node.dispatch(&mut cpass);
    }

    app.queue.submit(iter::once(encoder.finish()));
    app.device.poll(wgpu::MaintainBase::Wait);

    println!("input: {:?}", input_data);
    computer_node.read_output_buffer();
}
