use gol::WgpuState;

use imgui::*;
use imgui_wgpu::*;
use imgui_winit_support::*;

use wgpu::*;

use std::rc::Rc;

use crate::grid_drawer::*;
use crate::settings::*;

struct U32CharsFilter;
impl InputTextCallbackHandler for U32CharsFilter {
    fn char_filter(&mut self, c: char) -> Option<char> {
        if "1234567890".chars().any(|character| character == c) {
            Some(c)
        } else {
            None
        }
    }
}

pub struct Gui {
    device: Rc<Device>,
    queue: Rc<Queue>,
    context: Context,
    platform: WinitPlatform,
    renderer: Renderer,
    grid_columns: String,
    grid_rows: String,
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
            grid_columns: String::new(),
            grid_rows: String::new(),
            last_cursor: None,
        }
    }

    pub fn draw(
        &mut self,
        window: &winit::window::Window,
        surface_texture: &SurfaceTexture,
        grid: &mut GridDrawer,
        settings: &mut Settings,
    ) {
        self.platform
            .prepare_frame(self.context.io_mut(), &window)
            .expect("Fatal error: failed to prepare frame");

        let ui = self.context.frame();

        {
            let mut sqcolor_off = settings.sqcolor_off().to_f32();
            let mut sqcolor_on = settings.sqcolor_on().to_f32();
            let mut backgnd_clr = settings.background_color().to_f32();

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
                    // --COLORS-- \\s

                    ui.text("Square Color When Off");
                    let changed = ColorEdit::new("On Squares Color Editing", &mut sqcolor_off)
                        .label(false)
                        .build(&ui);

                    if changed {
                        settings.set_sqcolor_off(RGBA::from_f32(sqcolor_off));
                        grid.set_square_color_off(sqcolor_off);
                    }

                    ui.text("Square Color When On");
                    let changed = ColorEdit::new("Off Squares Color Editing", &mut sqcolor_on)
                        .label(false)
                        .build(&ui);

                    if changed {
                        settings.set_sqcolor_on(RGBA::from_f32(sqcolor_on));
                        grid.set_square_color_on(sqcolor_on);
                    }

                    ui.text("Background Color");
                    let changed = ColorEdit::new("Background Color Editing", &mut backgnd_clr)
                        .label(false)
                        .build(&ui);

                    if changed {
                        settings.set_background_color(RGBA::from_f32(backgnd_clr));
                    }

                    ui.separator();

                    // --GRID SIZE-- \\

                    ui.text("Grid X Dimension");
                    let x_changed = InputText::new(&ui, "Cols", &mut self.grid_columns)
                        .hint("Columns")
                        .enter_returns_true(true)
                        .callback(InputTextCallback::CHAR_FILTER, U32CharsFilter)
                        .build();

                    ui.text("Grid Y Dimension");
                    let y_changed = InputText::new(&ui, "Rows", &mut self.grid_rows)
                        .hint("Rows")
                        .enter_returns_true(true)
                        .callback(InputTextCallback::CHAR_FILTER, U32CharsFilter)
                        .build();

                    if x_changed || y_changed {
                        let old_x = settings.squares_x() as u32;
                        let old_y = settings.squares_y() as u32;

                        let mut new_x = old_x;
                        let mut new_y = old_y;

                        if let Ok(columns) = self.grid_columns.parse::<u32>() {
                            new_x = columns
                        }
                        if let Ok(rows) = self.grid_rows.parse::<u32>() {
                            new_y = rows
                        }

                        grid.resize_grid(new_x, new_y)
                    }
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
