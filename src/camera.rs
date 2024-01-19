use app_surface::AppSurface;
use std::f32::consts::FRAC_PI_2;
use std::time::Duration;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalPosition;
use winit::event::*;

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

/// “Camera”结构代表 3D 空间中的相机，具有位置、偏航和俯仰。
///
/// Properties:
///
/// * `position`: `position` 属性是一个 `glam::Vec3` 表示相机在 3D 空间中的位置。
/// * `yaw`: “yaw”属性表示围绕相机垂直轴的旋转。它决定相机的左右移动。
/// * `pitch`: `pitch` 属性表示相机的垂直旋转。它决定相机向上或向下倾斜的角度。
#[derive(Debug)]
pub struct Camera {
    pub position: glam::Vec3,
    yaw: f32,
    pitch: f32,
}

impl Camera {
    /// 函数“new”创建一个具有给定位置、偏航和俯仰的结构体的新实例。
    ///
    /// Arguments:
    ///
    /// * `position`: “position”参数是对象在 3D 空间中的初始位置。它的类型为“V”，这是一个可以转换为“glam::Vec3”类型的通用类型。这允许您传入任何可以转换为`的类型
    /// * `yaw`: “yaw”参数表示围绕垂直轴（通常是 y 轴）的旋转角度。它确定对象水平面向的方向。
    /// * `pitch`: `pitch`参数表示绕x轴的旋转，它决定了相机的垂直角度。它以度为单位进行测量，并使用“to_radians()”方法转换为弧度。
    ///
    /// Returns:
    ///
    /// “new”函数返回定义它的结构的实例。
    pub fn new<V: Into<glam::Vec3>>(position: V, yaw: f32, pitch: f32) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.to_radians(),
            pitch: pitch.to_radians(),
        }
    }

    /// `calc_matrix` 函数根据位置、俯仰和偏航值计算 4x4 矩阵。
    pub fn calc_matrix(&self) -> glam::Mat4 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        glam::Mat4::look_to_rh(
            self.position,
            glam::Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            glam::Vec3::Y,
        )
    }
}

/// “Projection”结构表示 Rust 中的投影矩阵，具有纵横比、视野、近剪裁平面和远剪裁平面的属性。
///
/// Properties:
///
/// * `aspect`: 投影的纵横比。它是投影的宽度与投影的高度的比率。
/// * `fovy`: fovy 属性表示垂直方向的视野角。它指定场景的垂直可见程度。
/// * `znear`: `znear` 属性表示到投影的近裁剪平面的距离。它决定了物体在开始被剪裁或从视图中消失之前与相机的距离有多近。
/// * `zfar`: “Projection”结构中的“zfar”属性表示从观看者到远裁剪平面的距离。它定义了场景中对象可见的最大距离。任何超出此距离的对象都将被剪裁并且不会渲染。
pub struct Projection {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Projection {
    /// 该函数创建一个具有指定宽度、高度、视野以及近剪裁平面和远剪裁平面的结构体的新实例。
    ///
    /// Arguments:
    ///
    /// * `width`: 视口的宽度（以像素为单位）。
    /// * `height`: “height”参数表示视口的高度或屏幕的高度（以像素为单位）。
    /// * `fovy`: “fovy”参数表示垂直方向的视野角，以度为单位。它决定通过相机可以看到多少场景。
    /// * `znear`: `znear` 参数表示到近裁剪平面的距离。它指定物体可见时距相机的最小距离。距离相机比“znear”更近的对象将被剪裁并且不会渲染。
    /// * `zfar`: `zfar` 参数表示不再渲染对象的距相机的距离。超出此距离的对象将被剪切并且在渲染场景中不可见。
    ///
    /// Returns:
    ///
    /// “new”函数返回定义它的结构的实例。
    pub fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.to_radians(),
            znear,
            zfar,
        }
    }

    /// `resize` 函数接受宽度和高度作为参数并更新对象的纵横比。
    ///
    /// Arguments:
    ///
    /// * `width`: width 参数的类型为 u32，这意味着它是一个无符号 32 位整数。它表示调整大小的新宽度值。
    /// * `height`: “height”参数是正在调整大小的对象或图像的所需高度。
    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    /// `calc_matrix` 函数使用给定的视场、纵横比、近平面距离和远平面距离返回透视投影矩阵。
    pub fn calc_matrix(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

/// “CameraController”结构表示 3D 环境中相机的控制器，具有各种移动和旋转属性。
///
/// Properties:
///
/// * `amount_left`: 一个浮点值，表示相机向左移动的量。
/// * `amount_right`: 向正确方向移动的量。
/// * `amount_forward`: 向前移动的量。
/// * `amount_backward`: “amount_backward”属性表示向后移动的量。它的类型为“f32”，这意味着它是一个 32 位浮点数。
/// * `amount_up`: 相机向上移动的量。
/// * `amount_down`: “amount_down”属性表示相机向下移动的量。它是一个“f32”（32 位浮点）值。
/// * `rotate_horizontal`: “rotate_horizontal”属性表示应用于相机的水平旋转量。它决定相机绕其垂直轴向左或向右旋转的程度。
/// * `rotate_vertical`: “rotate_vertical”属性表示应用于相机的垂直旋转量。它决定相机向上或向下倾斜的程度。
/// * `scroll`: `scroll` 属性表示接收到的滚动输入的数量。它的类型为“f32”，这意味着它是一个浮点数。
/// * `speed`: speed 属性决定相机在场景中移动的速度。
/// * `sensitivity`: 灵敏度属性决定相机控制器对用户输入的敏感程度。它会影响相机响应用户操作而旋转或移动的程度。
#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    /// 函数“new”使用各种移动和旋转量以及速度和灵敏度参数的默认值初始化一个结构体。
    ///
    /// Arguments:
    ///
    /// * `speed`: “speed”参数表示受此输入控制的对象或角色的移动速度。它确定对象响应输入命令的移动速度。
    /// * `sensitivity`: 灵敏度参数决定输入的灵敏度。对于给定输入，较高的灵敏度值将导致较大的移动或旋转变化。
    ///
    /// Returns:
    ///
    /// “new”函数返回定义它的结构的实例。
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    /// 函数“process_keyboard”将按键及其状态作为输入，并根据按下的按键更新相应的移动变量。
    ///
    /// Arguments:
    ///
    /// * `key`: ‘key’参数代表按下或释放的键盘按键的虚拟键码。
    /// * `state`: `state` 参数的类型为 `ElementState`，表示按键的状态，无论是按下还是释放。
    ///
    /// Returns:
    ///
    /// 一个布尔值，表示按键是否被处理。
    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.amount_forward = amount;
                true
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.amount_backward = amount;
                true
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.amount_left = amount;
                true
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.amount_right = amount;
                true
            }
            VirtualKeyCode::Space => {
                self.amount_up = amount;
                true
            }
            VirtualKeyCode::LShift => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    /// 函数“process_mouse”根据鼠标移动更新水平和垂直旋转值。
    ///
    /// Arguments:
    ///
    /// * `mouse_dx`: `mouse_dx` 参数表示鼠标 x 坐标的变化。它的类型为“f64”，这意味着它是一个双精度浮点数。
    /// * `mouse_dy`: `mouse_dy` 参数表示鼠标垂直位置的变化。它是一个“f64”（64位浮点）值，这意味着它可以存储十进制数。
    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    /// 函数“process_scroll”接受“MouseScrollDelta”并根据滚动增量的类型更新“scroll”变量。
    ///
    /// Arguments:
    ///
    /// * `delta`: 对 MouseScrollDelta 枚举的引用，表示鼠标的滚动移动。
    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = match delta {
            MouseScrollDelta::LineDelta(_, scroll) => -scroll * 2.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => -*scroll as f32,
        };
    }

    /// 该函数接受对“Camera”结构的可变引用和“Duration”作为输入。
    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        let forward = glam::Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = glam::Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        let (pitch_sin, pitch_cos) = camera.pitch.sin_cos();
        let scrollward =
            glam::Vec3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        camera.yaw += self.rotate_horizontal * self.sensitivity * dt;
        camera.pitch += -self.rotate_vertical * self.sensitivity * dt;

        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        if camera.pitch < -SAFE_FRAC_PI_2 {
            camera.pitch = -SAFE_FRAC_PI_2;
        } else if camera.pitch > SAFE_FRAC_PI_2 {
            camera.pitch = SAFE_FRAC_PI_2;
        }
    }
}

/// “CameraUniform”类型表示图形应用程序中相机的统一数据。
///
/// Properties:
///
/// * `view_position`: f32 值的 4 元素数组，表示相机在视图空间中的位置。这些元素对应于位置的 x、y、z 和 w 坐标。
/// * `view_proj`: “view_proj”属性是一个 4x4 矩阵，表示相机的组合视图和投影矩阵。它用于在 3D 渲染管道中将世界坐标转换为屏幕坐标。
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    /// 函数 update_view_proj 根据相机和投影参数更新视图位置和视图投影矩阵。
    ///
    /// Arguments:
    ///
    /// * `camera`: “camera”参数是“Camera”结构的一个实例。它表示摄像机在场景中的位置和方向。
    /// * `projection`: “projection”参数是“Projection”结构的一个实例。它表示用于将 3D 坐标转换为 2D 空间的投影矩阵。
    /// “Projection”结构的“calc_matrix()”方法以“Matrix4”类型返回投影矩阵。
    pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        self.view_position = camera.position.extend(1.0).into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).to_cols_array_2d()
    }
}

pub struct CameraState {
    pub camera: Camera,
    pub projection: Projection,
    pub camera_controller: CameraController,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group: wgpu::BindGroup,
    pub mouse_pressed: bool,
}

impl CameraState {
    /// 该函数在 Rust 应用程序中初始化相机及其关联的缓冲区和绑定组。
    ///
    /// Arguments:
    ///
    /// * `app`: “app”参数的类型为“&AppSurface”，它可能是对应用程序表面或窗口的引用。用于访问设备并创建与相机相关的各种资源。
    ///
    /// Returns:
    ///
    /// “new”函数返回定义它的结构的实例。
    pub fn new(app: &AppSurface) -> Self {
        let camera = Camera::new((15.0, 5.0, 15.0), -90.0, -20.0);
        let projection = Projection::new(app.config.width, app.config.height, 45.0, 0.1, 100.0);
        let camera_controller = CameraController::new(4.0, 0.4);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);
        let camera_buffer = app
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera_bind_group_layout =
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
                    label: Some("camera_bind_group_layout"),
                });
        let camera_bind_group = app.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });
        let mouse_pressed = false;
        Self {
            camera,
            projection,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
            mouse_pressed,
        }
    }
    /// 此函数更新 Rust 应用程序中的相机和相机制服。
    ///
    /// Arguments:
    ///
    /// * `app`: `app` 参数的类型为 `AppSurface`。它代表将在其上渲染图形的应用程序表面或窗口。
    /// * `dt`: `dt` 是一个 `std::time::Duration` 参数，表示当前帧和前一帧之间的时间差。它用于根据经过的时间更新相机的位置和方向。
    pub fn update(&mut self, app: &AppSurface, dt: std::time::Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform
            .update_view_proj(&self.camera, &self.projection);
        app.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    /// 该函数处理各种输入事件，例如键盘输入、鼠标滚轮滚动和鼠标按钮单击。
    ///
    /// Arguments:
    ///
    /// * `event`: “event”参数的类型为“WindowEvent”，表示窗口上发生的事件。
    ///
    /// Returns:
    ///
    /// 一个布尔值，表示事件是否被处理。
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => self.camera_controller.process_keyboard(*key, *state),
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }
}
