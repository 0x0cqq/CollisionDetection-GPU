use anyhow::*;
use image::GenericImageView;

/// “Texture”结构表示 Rust 中的纹理以及相关属性，例如纹理本身、纹理视图和采样器。
///
/// Properties:
///
/// * `texture`: 表示实际纹理数据的 wgpu::Texture 对象。
/// * `view`: “view”属性是一个“wgpu::TextureView”，它是“wgpu::Texture”的视图。它允许您访问纹理数据并对其执行操作，例如读取或写入像素。
/// * `sampler`:
/// “sampler”属性是“wgpu::Sampler”类型的对象。采样器用于控制在渲染期间访问纹理元素（纹理元素）时如何对纹理进行采样。它定义了过滤模式、寻址模式和边框颜色等属性。
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    /// `create_depth_texture` 函数使用给定设备创建具有指定配置的深度纹理。
    ///
    /// Arguments:
    ///
    /// * `device`: 对 wgpu::Device 对象的引用，它表示用于渲染的 GPU 设备。
    /// * `config`: `config` 参数的类型为 `&wgpu::SurfaceConfiguration`，表示将渲染深度纹理的表面的配置。它包含表面的宽度和高度等信息。
    /// * `label`: 为深度纹理提供标签的字符串。该标签用于在 GPU 调试器和分析器工具中调试和识别纹理。
    ///
    /// Returns:
    ///
    /// 包含深度纹理、其视图和采样器的结构体实例。
    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 200.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }

    /// Rust 中的“from_bytes”函数接受设备、队列、字节、标签和指示它是否是法线映射的布尔标志，并返回结果。
    ///
    /// Arguments:
    ///
    /// * `device`: 对 wgpu::Device 对象的引用，该对象表示将用于图像操作的 GPU 设备。
    /// * `queue`: `queue` 参数是 `wgpu::Queue` 的实例，它表示用于向设备提交 GPU 命令的命令队列。
    /// * `bytes`: 表示图像数据的字节片。
    /// * `label`: 表示纹理的标签或名称的字符串。它用于调试和识别纹理。
    /// * `is_normal_map`: 指示图像是否为法线贴图的布尔值。
    ///
    /// Returns:
    ///
    /// 一个 `Result<Self>`。
    #[allow(dead_code)]
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
        is_normal_map: bool,
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, Some(label), is_normal_map)
    }

    /// 函数获取图像并从中创建纹理，以及视图和采样器。
    ///
    /// Arguments:
    ///
    /// * `device`: 对 wgpu::Device 的引用，表示用于渲染的 GPU 设备。
    /// * `queue`: `queue` 是对 `wgpu::Queue` 对象的引用。它用于将命令提交给GPU执行。
    /// * `img`: “img”参数的类型为“image::DynamicImage”，它表示可以从各种格式（例如 PNG、JPEG、BMP）加载的图像。它包含图像的像素数据和其他元数据。
    /// * `label`: 纹理的可选标签。它可用于调试或识别目的。
    /// * `is_normal_map`: `is_normal_map`
    /// 参数是一个布尔标志，指示图像是否应被视为法线贴图。如果“is_normal_map”为“true”，则用于纹理的图像格式将为“wgpu::TextureFormat::Rgba8Unorm”，它表示法线
    ///
    /// Returns:
    ///
    /// a `Result<Self>`，其中 `Self` 指的是定义函数的结构类型。
    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
        is_normal_map: bool,
    ) -> Result<Self> {
        let dimensions = img.dimensions();
        let rgba = img.to_rgba8();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: if is_normal_map {
                wgpu::TextureFormat::Rgba8Unorm
            } else {
                wgpu::TextureFormat::Rgba8UnormSrgb
            },
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }
}
