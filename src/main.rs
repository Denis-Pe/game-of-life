mod graphics;

use graphics::WgpuState;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    window.set_title("Conway's Game of Life");
    window.set_min_inner_size(Some(winit::dpi::LogicalSize {
        width: 800,
        height: 450,
    }));

    let mut wgpu_state = pollster::block_on(WgpuState::new(&window));

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            wgpu_state.update();
            match wgpu_state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => wgpu_state.resize(wgpu_state.size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(error) => eprintln!("{:?}", error),
            }
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }

        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !wgpu_state.input(event) {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        wgpu_state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        wgpu_state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    });
}
