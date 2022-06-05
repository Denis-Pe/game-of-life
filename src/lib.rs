use wgpu::*;

use winit::event_loop::ControlFlow;
use winit::window::Window;

use std::rc::Rc;

// Why Reference Counters?
// I need shared ownership because
// 1. Those three fields will be flying around
// multiple structs, and
// 2. A normal reference will not do
// apparently because of behind-the-scenes
// stuff Rust does with closures, but with
// shared ownership Rust can do everything
// it wants but the memory stays on the heap
// and it gets cleaned up at the end of the last
// owner's lifetime
pub struct WgpuState {
    pub surface: Rc<Surface>,
    pub device: Rc<Device>,
    pub queue: Rc<Queue>,
    pub config: SurfaceConfiguration,
}

impl WgpuState {
    pub async fn new(window: &Window) -> WgpuState {
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

        Self {
            surface: Rc::new(surface),
            device: Rc::new(device),
            queue: Rc::new(queue),
            config,
        }
    }

    pub fn resize_window(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn clear_screen(
        &self,
        color: Color,
        surface_texture: &SurfaceTexture,
    ) -> Result<(), SurfaceError> {
        // let surface_texture = self.surface.get_current_texture()?;

        let view = surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
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

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    pub fn handle_render_result(
        &mut self,
        result: Result<(), SurfaceError>,
        control_flow: &mut ControlFlow,
        window: &Window,
    ) {
        match result {
            Ok(_) => {}
            Err(SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
            Err(SurfaceError::Lost) => self.resize_window(window.inner_size()),
            Err(error) => eprintln!("{:?}", error),
        }
    }
}
