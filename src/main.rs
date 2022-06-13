mod grid_drawer;
use grid_drawer::*;

mod gui;
use gui::*;

mod settings;
use settings::*;

use winit::{
    dpi::PhysicalPosition,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use wgpu::*;

use gol::*;

const TRANSLATION_CONSTANT: f32 = 0.001;
const ZOOMING_CONSTANT: f32 = 0.05;

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

    let mut settings = Settings::default();

    let mut grid = GridDrawer::new(&wgpu_state, &settings);

    let mut gui = Gui::new(&window, &wgpu_state);

    let mut last_cursor: Option<PhysicalPosition<f64>> = None;
    let mut mouse_held = false;
    let mut control = false;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                match wgpu_state.surface.get_current_texture() {
                    Ok(surface_texture) => {
                        let mut results = Vec::new();

                        let backgnd_clr = settings.background_color().to_f32();
                        results.push(wgpu_state.clear_screen(
                            Color {
                                r: backgnd_clr[0] as f64,
                                g: backgnd_clr[1] as f64,
                                b: backgnd_clr[2] as f64,
                                a: backgnd_clr[3] as f64,
                            },
                            &surface_texture,
                        ));

                        results.push(grid.draw(&surface_texture));

                        gui.draw(&window, &surface_texture, &mut grid, &mut settings);

                        if !results.iter().any(|result| result.is_err()) {
                            surface_texture.present()
                        } else {
                            for result in results.into_iter() {
                                wgpu_state.handle_render_result(result, control_flow, &window)
                            }
                        }
                    }

                    Err(error) => {
                        wgpu_state.handle_render_result(Err(error), control_flow, &window)
                    }
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

                WindowEvent::MouseInput { state, button, .. } => {
                    if let MouseButton::Left = button {
                        if let ElementState::Pressed = state {
                            mouse_held = true
                        } else {
                            mouse_held = false
                        }
                    }
                }

                WindowEvent::MouseWheel { delta, .. } => {
                    if let MouseScrollDelta::LineDelta(_, y) = delta {
                        grid.change_grid_zoom(y * ZOOMING_CONSTANT);
                    }
                }

                WindowEvent::CursorMoved {
                    position: new_position,
                    ..
                } => {
                    if let Some(position) = last_cursor {
                        let delta = PhysicalPosition {
                            x: (new_position.x - position.x) as f32,
                            y: (new_position.y - position.y) as f32,
                        };

                        if mouse_held {
                            let zoom_multiplier = if grid.grid_zoom() > 1.0 {
                                1.0
                            } else {
                                1.0 / grid.grid_zoom()
                            };

                            grid.translate_grid([
                                zoom_multiplier * delta.x * TRANSLATION_CONSTANT,
                                zoom_multiplier * -delta.y * TRANSLATION_CONSTANT,
                            ]);
                        }
                    }
                    last_cursor = Some(*new_position);
                }

                WindowEvent::ModifiersChanged(state) => control = state.ctrl(),

                WindowEvent::KeyboardInput { input, .. } if control => {
                    if let Some(VirtualKeyCode::R) = input.virtual_keycode {
                        grid.set_grid_zoom(1.0);
                        grid.set_grid_translation([0.0, 0.0]);
                    }
                }

                _ => {}
            },

            _ => {}
        }

        gui.platform_event_handling(&window, &event)
    });
}
