use gol::WgpuState;

use imgui::*;
use imgui_wgpu::*;
use imgui_winit_support::*;

use wgpu::*;

use std::rc::Rc;

use crate::grid_drawer::*;
use crate::settings::*;

pub struct Gui {
    device: Rc<Device>,
    queue: Rc<Queue>,
    context: Context,
    platform: WinitPlatform,
    renderer: Renderer,
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
            last_cursor: None,
        }
    }

    /// Returns whether any of the colors were changed or not
    fn color_widgets(ui: &Ui, grid: &mut GridDrawer, settings: &mut Settings) -> bool {
        let mut sqcolor_off = settings.sqcolor_off().to_f32();
        let mut sqcolor_on = settings.sqcolor_on().to_f32();
        let mut backgnd_clr = settings.background_color().to_f32();

        let mut overall_change = false;

        ui.text("Square Color When Off");
        let changed = ColorEdit::new("On Squares Color Editing", &mut sqcolor_off)
            .label(false)
            .alpha_bar(true)
            .build(&ui);
        if changed {
            settings.set_sqcolor_off(RGBA::from_f32(sqcolor_off));
            grid.set_square_color_off(sqcolor_off);
            overall_change = true;
        }

        ui.text("Square Color When On");
        let changed = ColorEdit::new("Off Squares Color Editing", &mut sqcolor_on)
            .label(false)
            .alpha_bar(true)
            .build(&ui);
        if changed {
            settings.set_sqcolor_on(RGBA::from_f32(sqcolor_on));
            grid.set_square_color_on(sqcolor_on);
            overall_change = true;
        }

        ui.text("Background Color");
        let changed = ColorEdit::new("Background Color Editing", &mut backgnd_clr)
            .label(false)
            .alpha(false)
            .build(&ui);
        if changed {
            settings.set_background_color(RGBA::from_f32(backgnd_clr));
            overall_change = true;
        }

        overall_change
    }

    fn grid_dimensions_widgets(ui: &Ui, grid: &mut GridDrawer, settings: &mut Settings) {
        let mut columns = settings.squares_x() as i32;
        let mut rows = settings.squares_y() as i32;

        ui.text("Grid X Dimension");
        InputInt::new(&ui, "Cols", &mut columns)
            .enter_returns_true(true)
            .build();
        if columns <= 4 {
            columns = 5
        }

        ui.text("Grid Y Dimension");
        InputInt::new(&ui, "Rows", &mut rows)
            .enter_returns_true(true)
            .build();
        if rows <= 4 {
            rows = 5
        }

        grid.resize_grid(columns as u32, rows as u32);
        settings.resize_grid(columns as u16, rows as u16);
    }

    /// Returns whether the colors were changed
    pub fn draw(
        &mut self,
        window: &winit::window::Window,
        surface_texture: &SurfaceTexture,
        grid: &mut GridDrawer,
        settings: &mut Settings,
    ) -> bool {
        self.platform
            .prepare_frame(self.context.io_mut(), &window)
            .expect("Fatal error: failed to prepare frame");

        let ui = self.context.frame();

        let mut colors_changed = false;

        {
            let left_panel = Window::new("is it you?!");
            left_panel
                .title_bar(false)
                .position([0.0, 0.0], Condition::Always)
                .size(
                    [200.0, window.inner_size().height as f32],
                    Condition::Always,
                )
                .movable(false)
                .resizable(false)
                .build(&ui, || {
                    colors_changed = Self::color_widgets(&ui, grid, settings);

                    ui.separator();

                    Self::grid_dimensions_widgets(&ui, grid, settings);

                    ui.separator();
                });
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

        colors_changed
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
