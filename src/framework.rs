use super::State;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub fn run(wh_ratio: Option<f32>) {
    env_logger::init();

    let (event_loop, instance) = pollster::block_on(create_action_instance(wh_ratio));
    start_event_loop(event_loop, instance);
}


async fn create_action_instance(wh_ratio: Option<f32>) -> (EventLoop<()>, State) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // 计算一个默认显示高度
    let height = (if cfg!(target_arch = "wasm32") {
        550.0
    } else {
        600.0
    } * window.scale_factor()) as u32;

    let width = if let Some(ratio) = wh_ratio {
        (height as f32 * ratio) as u32
    } else {
        height
    };
    window.set_inner_size(PhysicalSize::new(width, height));

    let app = app_surface::AppSurface::new(window).await;
    let instance = State::new(app).await;

    let adapter_info = instance.get_adapter_info();
    let gpu_info = format!(
        "正在使用 {}, 后端图形接口为 {:?}。",
        adapter_info.name, adapter_info.backend
    );
    println!("{gpu_info}");


    (event_loop, instance)
}

fn start_event_loop(event_loop: EventLoop<()>, state: State) {
    let mut state = state;
    let mut last_render_time = instant::Instant::now();
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                if state.camera_state.mouse_pressed {
                    state.camera_state.camera_controller.process_mouse(delta.0, delta.1)
                }
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.current_window_id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            if physical_size.width == 0 || physical_size.height == 0 {
                                // 处理最小化窗口的事件
                                println!("Window minimized!");
                            } else {
                                state.resize(physical_size);
                            }
                        }
                        WindowEvent::ScaleFactorChanged {
                            scale_factor: _,
                            new_inner_size,
                        } => {
                            state.resize(new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.current_window_id() => {
                let now = instant::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                state.update(dt);

                match state.render() {
                    Ok(_) => {}
                    // 当展示平面的上下文丢失，就需重新配置
                    Err(wgpu::SurfaceError::Lost) => eprintln!("Surface is lost"),
                    // 系统内存不足时，程序应该退出。
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // 所有其他错误（过期、超时等）应在下一帧解决
                    Err(e) => eprintln!("{e:?}"),
                }
            }
            Event::MainEventsCleared => {
                // 除非我们手动请求，RedrawRequested 将只会触发一次。
                state.request_redraw();
            }
            _ => {}
        }
    });
}