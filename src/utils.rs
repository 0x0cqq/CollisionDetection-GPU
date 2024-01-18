pub fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("{shader:?}")),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format.add_srgb_suffix(),
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        // If the pipeline will be used with a multiview render pass, this
        // indicates how many array layers the attachments will have.
        multiview: None,
    })
}

pub fn bytes_to_u32(bytes: &[u8]) -> Vec<u32> {
    let mut results: Vec<u32> = Vec::new();

    for i in 0..bytes.len() / 4 {
        let value = u32::from_ne_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap());
        results.push(value);
    }

    results
}

pub fn bytes_to_f32(bytes: &[u8]) -> Vec<f32> {
    let mut results: Vec<f32> = Vec::new();

    for i in 0..bytes.len() / 4 {
        let value = f32::from_ne_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap());
        results.push(value);
    }

    results
}

pub fn output_bytes_as_u32(bytes: &[u8], label: &str) {
    println!("Label: {:?} Output: {:?}", label, bytes_to_u32(bytes));
}

pub fn output_bytes_as_f32(bytes: &[u8], label: &str) {
    println!("Label: {:?} Output: {:?}", label, bytes_to_f32(bytes));
}
