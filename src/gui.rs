use egui_wgpu_backend::RenderPass;
use egui_winit_platform::{Platform, PlatformDescriptor};
use gol::*;

pub struct Egui {
    platform: Platform,
    render_pass: egui_wgpu_backend::RenderPass,
}

impl Egui {
    pub fn new(
        window: &winit::window::Window,
        wgpu_state: &WgpuState,
        font_definitions: egui::FontDefinitions,
        style: egui::Style,
    ) -> Self {
        let platform = Platform::new(PlatformDescriptor {
            physical_width: window.inner_size().width,
            physical_height: window.inner_size().height,
            scale_factor: window.scale_factor(),
            font_definitions,
            style,
        });

        let render_pass = RenderPass::new(
            wgpu_state.device.as_ref(),
            wgpu_state
                .surface
                .get_preferred_format(wgpu_state.adapter.as_ref())
                .unwrap(),
            1,
        );

        Self {
            platform,
            render_pass,
        }
    }
}
