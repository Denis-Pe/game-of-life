use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use wgpu::*;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    window.set_title("Conway's Game of Life");
    window.set_min_inner_size(Some(winit::dpi::LogicalSize {
        width: 800,
        height: 450,
    }));

    let (surface, device, queue, mut configuration) = pollster::block_on(init_wgpu(&window));

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            match clear_screen(
                Color {
                    r: 0.01,
                    g: 0.01,
                    b: 0.1,
                    a: 1.0,
                },
                &surface,
                &device,
                &queue,
            ) {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => {
                    resize(&mut configuration, &surface, &device, window.inner_size())
                }
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
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {
                resize(&mut configuration, &surface, &device, *physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                resize(&mut configuration, &surface, &device, **new_inner_size);
            }
            _ => {}
        },

        _ => {}
    });
}

async fn init_wgpu(
    window: &winit::window::Window,
) -> (Surface, Device, Queue, SurfaceConfiguration) {
    let instance = Instance::new(Backends::all());
    let surface = unsafe { instance.create_surface(window) };
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                label: None,
                features: Features::empty(),
                limits: Limits::default(),
            },
            None,
        )
        .await
        .unwrap();

    let config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_preferred_format(&adapter).unwrap(),
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: PresentMode::Fifo,
    };
    surface.configure(&device, &config);

    (surface, device, queue, config)
}

fn resize(
    config: &mut SurfaceConfiguration,
    surface: &Surface,
    device: &Device,
    new_size: winit::dpi::PhysicalSize<u32>,
) {
    if new_size.width > 0 && new_size.height > 0 {
        config.width = new_size.width;
        config.height = new_size.height;
        surface.configure(device, config);
    }
}

fn clear_screen(
    color: Color,
    surface: &Surface,
    device: &Device,
    queue: &Queue,
) -> Result<(), SurfaceError> {
    let output = surface.get_current_texture()?;

    let view = output
        .texture
        .create_view(&TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("render_encoder"),
    });

    let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some("render_pass"),
        color_attachments: &[RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(color),
                store: true,
            },
        }],
        depth_stencil_attachment: None,
    });

    drop(render_pass);

    queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
}
