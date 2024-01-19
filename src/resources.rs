use std::io::{BufReader, Cursor};

use wgpu::util::DeviceExt;

use crate::{model, texture};

/// `load_string` 函数将文件内容作为 Rust 中的字符串加载。
///
/// Arguments:
///
/// * `file_name`: `file_name` 参数是一个字符串，表示要加载的文件的名称。
///
/// Returns:
///
/// 函数“load_string”返回“Result”类型，成功情况包含“String”，错误情况包含“anyhow::Error”。
pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(file_name);
    let txt = std::fs::read_to_string(path)?;

    Ok(txt)
}

/// 函数“load_binary”加载二进制文件并将其内容作为字节向量返回。
///
/// Arguments:
///
/// * `file_name`: `file_name` 参数是一个字符串，表示要作为二进制文件加载的文件的名称。
///
/// Returns:
///
/// 函数“load_binary”返回“Result”类型，成功情况包含“Vec<u8>”（字节向量），错误情况包含“anyhow::Error”。
pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(file_name);
    let data = std::fs::read(path)?;

    Ok(data)
}

/// 函数“load_texture”从文件加载纹理并返回包含加载纹理的“Result”。
///
/// Arguments:
///
/// * `file_name`: 包含纹理数据的文件的名称。
/// * `is_normal_map`: 一个布尔值，指示纹理是否是法线贴图。
/// * `device`: `device` 参数是 `wgpu::Device` 的实例，它代表将用于创建和管理资源的 GPU 设备。
/// * `queue`: `queue` 参数是 `wgpu::Queue` 的实例，它表示用于向设备提交 GPU 命令的命令队列。用于向GPU提交纹理加载命令进行处理。
///
/// Returns:
///
/// 一个“Result”类型，其中“Texture”结构作为成功变量，“anyhow::Error”作为错误变量。
pub async fn load_texture(
    file_name: &str,
    is_normal_map: bool,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<texture::Texture> {
    println!("Loading texture {:?}", file_name);
    let data = load_binary(file_name).await?;
    texture::Texture::from_bytes(device, queue, &data, file_name, is_normal_map)
}

/// Rust 中的“load_model”函数从文件加载 3D 模型，包括其材质和纹理，并使用网格和材质创建模型对象。
///
/// Arguments:
///
/// * `file_name`: 包含模型数据的文件的名称。
/// * `device`: 对 wgpu::Device 的引用，表示用于渲染的 GPU 设备。
/// * `queue`: `queue` 参数是 `wgpu::Queue` 的实例，它代表用于提交 GPU 命令的命令队列。它用于将命令提交给GPU进行处理。
/// * `layout`: `layout` 参数是对 `wgpu::BindGroupLayout` 对象的引用。该对象定义用于将资源（例如纹理）绑定到着色器管道的绑定组的布局。它用于为模型创建材料。
/// * `scale_factor`: “scale_factor”参数是一个浮点值，用于确定应用于模型顶点的缩放因子。它用于将模型调整到所需的尺寸。
///
/// Returns:
///
/// 如果加载过程成功，函数“load_model”将返回一个包含“model::Model”对象的“Result”。
pub async fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    scale_factor: f32,
) -> anyhow::Result<model::Model> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = load_string(&p).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let mut materials = Vec::new();
    for m in obj_materials? {
        let diffuse_texture = load_texture(&m.diffuse_texture, false, device, queue).await?;
        let normal_texture = load_texture(&m.normal_texture, true, device, queue).await?;

        materials.push(model::Material::new(
            device,
            &m.name,
            diffuse_texture,
            normal_texture,
            layout,
        ));
    }

    let meshes = models
        .into_iter()
        .map(|m| {
            let mut vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| model::ModelVertex {
                    position: [
                        m.mesh.positions[i * 3] * scale_factor,
                        m.mesh.positions[i * 3 + 1] * scale_factor,
                        m.mesh.positions[i * 3 + 2] * scale_factor,
                    ],
                    tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                    // We'll calculate these later
                    tangent: [0.0; 3],
                    bitangent: [0.0; 3],
                })
                .collect::<Vec<_>>();

            let indices = &m.mesh.indices;
            let mut triangles_included = vec![0; vertices.len()];

            // Calculate tangents and bitangets. We're going to
            // use the triangles, so we need to loop through the
            // indices in chunks of 3
            for c in indices.chunks(3) {
                let v0 = vertices[c[0] as usize];
                let v1 = vertices[c[1] as usize];
                let v2 = vertices[c[2] as usize];

                let pos0: glam::Vec3 = v0.position.into();
                let pos1: glam::Vec3 = v1.position.into();
                let pos2: glam::Vec3 = v2.position.into();

                let uv0: glam::Vec2 = v0.tex_coords.into();
                let uv1: glam::Vec2 = v1.tex_coords.into();
                let uv2: glam::Vec2 = v2.tex_coords.into();

                // Calculate the edges of the triangle
                let delta_pos1 = pos1 - pos0;
                let delta_pos2 = pos2 - pos0;

                // This will give us a direction to calculate the
                // tangent and bitangent
                let delta_uv1 = uv1 - uv0;
                let delta_uv2 = uv2 - uv0;

                // Solving the following system of equations will
                // give us the tangent and bitangent.
                //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
                //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
                // Luckily, the place I found this equation provided
                // the solution!
                let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                // We flip the bitangent to enable right-handed normal
                // maps with wgpu texture coordinate system
                let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

                // We'll use the same tangent/bitangent for each vertex in the triangle
                vertices[c[0] as usize].tangent =
                    (tangent + glam::Vec3::from_array(vertices[c[0] as usize].tangent)).into();
                vertices[c[1] as usize].tangent =
                    (tangent + glam::Vec3::from_array(vertices[c[1] as usize].tangent)).into();
                vertices[c[2] as usize].tangent =
                    (tangent + glam::Vec3::from_array(vertices[c[2] as usize].tangent)).into();
                vertices[c[0] as usize].bitangent =
                    (bitangent + glam::Vec3::from_array(vertices[c[0] as usize].bitangent)).into();
                vertices[c[1] as usize].bitangent =
                    (bitangent + glam::Vec3::from_array(vertices[c[1] as usize].bitangent)).into();
                vertices[c[2] as usize].bitangent =
                    (bitangent + glam::Vec3::from_array(vertices[c[2] as usize].bitangent)).into();

                // Used to average the tangents/bitangents
                triangles_included[c[0] as usize] += 1;
                triangles_included[c[1] as usize] += 1;
                triangles_included[c[2] as usize] += 1;
            }

            // Average the tangents/bitangents
            for (i, n) in triangles_included.into_iter().enumerate() {
                let denom = 1.0 / n as f32;
                let v = &mut vertices[i];
                v.tangent = (glam::Vec3::from_array(v.tangent) * denom).into();
                v.bitangent = (glam::Vec3::from_array(v.bitangent) * denom).into();
            }

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{file_name:?} Vertex Buffer")),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{file_name:?} Index Buffer")),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            model::Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    Ok(model::Model { meshes, materials })
}
