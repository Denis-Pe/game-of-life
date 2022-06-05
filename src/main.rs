mod grid_drawer;
use grid_drawer::*;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use wgpu::*;

use gol::*;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Conway's Game of Life")
        .with_min_inner_size(winit::dpi::LogicalSize {
            width: 450u32,
            height: 450u32,
        })
        .build(&event_loop)
        .unwrap();

    let mut wgpu_state = pollster::block_on(WgpuState::new(&window));

    let grid = GridDrawer::new(&wgpu_state);

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            match wgpu_state.surface.get_current_texture() {
                Ok(surface_texture) => {
                    let mut results = Vec::new();

                    results.push(wgpu_state.clear_screen(
                        Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        },
                        &surface_texture,
                    ));

                    results.push(grid.draw(&surface_texture));

                    if !results.iter().any(|result| result.is_err()) {
                        surface_texture.present()
                    } else {
                        for result in results.into_iter() {
                            wgpu_state.handle_render_result(result, control_flow, &window)
                        }
                    }
                }

                Err(error) => wgpu_state.handle_render_result(Err(error), control_flow, &window),
            };
        }

        Event::MainEventsCleared => {
            window.request_redraw();
        }

        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            }

            WindowEvent::Resized(physical_size) => {
                wgpu_state.resize_window(*physical_size);
                grid.resize_window(*physical_size);
            }

            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                wgpu_state.resize_window(**new_inner_size);
                grid.resize_window(**new_inner_size);
            }

            _ => {}
        },

        _ => {}
    });
}
