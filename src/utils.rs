/// 该代码提供了辅助函数，用于在 Rust 中创建渲染管道并将字节转换为 u32 和 f32 值。
///
/// Arguments:
///
/// * `device`: 对 wgpu::Device 对象的引用，它表示用于渲染的 GPU 设备。
/// * `layout`: `layout` 参数是对 `wgpu::PipelineLayout` 对象的引用，它定义管道绑定组的布局。
/// * `color_format`: `color_format` 参数是渲染管道中颜色附件的格式。它指定如何存储和解释颜色数据。
/// * `depth_format`: `depth_format` 参数是一个可选的
/// `wgpu::TextureFormat`，它指定深度纹理的格式。如果提供了“Some(format)”，则将使用指定的格式为渲染管道创建深度模板状态。如果提供“None”，则没有深度
/// * `vertex_layouts`: `vertex_layouts` 参数是 `wgpu::VertexBufferLayout`
/// 结构的数组。每个结构体都描述渲染管道中使用的顶点缓冲区的布局。它指定步幅（每个顶点的大小，以字节为单位）、步长模式（顶点缓冲区是逐顶点还是
/// * `shader`:
/// “shader”参数是一个“wgpu::ShaderModuleDescriptor”，它描述渲染管道中使用的着色器模块。它包含诸如着色器代码以及顶点和片段着色器的入口点等信息。
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

/// 该代码提供了将字节切片转换为 u32 或 f32 值向量的函数，还包括使用标签打印转换后的值的函数。
///
/// Arguments:
///
/// * `bytes`: 表示数据序列的字节片。
///
/// Returns:
///
/// `bytes_to_u32` 函数返回一个 `Vec<u32>`，其中包含输入 `bytes` 切片的转换值。
pub fn bytes_to_u32(bytes: &[u8]) -> Vec<u32> {
    let mut results: Vec<u32> = Vec::new();

    for i in 0..bytes.len() / 4 {
        let value = u32::from_ne_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap());
        results.push(value);
    }

    results
}

/// 该代码提供了将字节数组转换为 f32 值向量的函数，并将字节数组打印为带标签的 u32 或 f32 值。
///
/// Arguments:
///
/// * `bytes`: 表示要转换为 f32 值的数据的字节片。每个 f32 值由切片中的 4 个字节表示。
///
/// Returns:
///
/// `bytes_to_f32` 函数返回一个 `Vec<f32>`，它是 32 位浮点数的向量。
#[allow(dead_code)]
pub fn bytes_to_f32(bytes: &[u8]) -> Vec<f32> {
    let mut results: Vec<f32> = Vec::new();

    for i in 0..bytes.len() / 4 {
        let value = f32::from_ne_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap());
        results.push(value);
    }

    results
}

#[allow(dead_code)]
pub fn output_bytes_as_u32(bytes: &[u8], label: &str) {
    println!("Label: {:?} Output: {:?}", label, bytes_to_u32(bytes));
}

#[allow(dead_code)]
pub fn output_bytes_as_f32(bytes: &[u8], label: &str) {
    println!("Label: {:?} Output: {:?}", label, bytes_to_f32(bytes));
}
