use gol::WgpuState;

use imgui::*;
use imgui_wgpu::*;
use imgui_winit_support::*;

use wgpu::*;

use std::rc::Rc;

pub struct Gui {
    device: Rc<Device>,
    queue: Rc<Queue>,
    context: Context,
    platform: WinitPlatform,
    renderer: Renderer,
    demo_open: bool,
    last_cursor: Option<MouseCursor>,
}

impl Gui {
    pub fn new(window: &winit::window::Window, wgpu_state: &WgpuState) -> Self {
        let mut context = Context::create();
        let mut platform = WinitPlatform::init(&mut context);

        platform.attach_window(context.io_mut(), window, HiDpiMode::Default);

        context.set_ini_filename(None);

        let hidpi_factor = window.scale_factor();

        let font_size = (13.0 * hidpi_factor) as f32;
        context.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        context.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: font_size,
                oversample_h: 1,
                pixel_snap_h: true,
                ..Default::default()
            }),
        }]);

        let renderer_config = RendererConfig {
            texture_format: wgpu_state.config.format,
            ..Default::default()
        };

        let renderer = Renderer::new(
            &mut context,
            wgpu_state.device.as_ref(),
            wgpu_state.queue.as_ref(),
            renderer_config,
        );

        Self {
            device: Rc::clone(&wgpu_state.device),
            queue: Rc::clone(&wgpu_state.queue),
            context,
            platform,
            renderer,
            demo_open: true,
            last_cursor: None,
        }
    }

    pub fn draw(&mut self, window: &winit::window::Window, surface_texture: &SurfaceTexture) {
        self.platform
            .prepare_frame(self.context.io_mut(), &window)
            .expect("Fatal error: failed to prepare frame");

        let ui = self.context.frame();

        {
            let window = Window::new("Hello world");
            window
                .size([300.0, 100.0], Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text("Hello world!");
                    ui.text("This...is...imgui-rs on WGPU!");
                    ui.separator();
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(format!(
                        "Mouse Position: ({:.1},{:.1})",
                        mouse_pos[0], mouse_pos[1]
                    ));
                });

            let window = Window::new("Hello too");
            window
                .size([400.0, 200.0], Condition::FirstUseEver)
                .position([400.0, 200.0], Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text("HELLO WORLD!");
                });

            ui.show_demo_window(&mut self.demo_open);
        }

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("imgui_command_encoder"),
            });

        if self.last_cursor != ui.mouse_cursor() {
            self.last_cursor = ui.mouse_cursor();
            self.platform.prepare_render(&ui, window);
        }

        let view = surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("imgui_render_pass"),
            color_attachments: &[RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        self.renderer
            .render(
                ui.render(),
                self.queue.as_ref(),
                self.device.as_ref(),
                &mut render_pass,
            )
            .expect("Fatal error: rendering failed");

        drop(render_pass);

        self.queue.submit(Some(encoder.finish()));
    }

    pub fn platform_event_handling(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::Event<()>,
    ) {
        self.platform
            .handle_event(self.context.io_mut(), window, event);
    }
}
