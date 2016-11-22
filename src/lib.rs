extern crate app_units;
extern crate webrender;
extern crate glutin;
extern crate gleam;
extern crate webrender_traits;
extern crate euclid;

use euclid::{Size2D, Point2D, Matrix4D};
use gleam::gl;
use std::thread;
use std::time;
use webrender_traits::{PipelineId};
use webrender_traits::{AuxiliaryListsBuilder, Epoch, ColorF, ClipRegion};
use webrender_traits::{RendererKind};

struct Notifier {
    window_proxy: glutin::WindowProxy,
}

impl Notifier {
    fn new(window_proxy: glutin::WindowProxy) -> Notifier {
        Notifier { window_proxy: window_proxy }
    }
}

impl webrender_traits::RenderNotifier for Notifier {
    fn new_frame_ready(&mut self) {
        self.window_proxy.wakeup_event_loop();
    }

    fn new_scroll_frame_ready(&mut self, _composite_needed: bool) {
        self.window_proxy.wakeup_event_loop();
    }

    fn pipeline_size_changed(&mut self, _: PipelineId, _: Option<Size2D<f32>>) {}
}

pub fn run<T: AsDisplayItem>(item: T) {
    let window = glutin::WindowBuilder::new()
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)))
        .build()
        .unwrap();

    unsafe {
        window.make_current().ok();
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
        gl::clear_color(0.3, 0.0, 0.0, 1.0);
    }

    let (width, height) = window.get_inner_size().unwrap();
    let window_size = Size2D::new(width as f32, height as f32);

    let opts = webrender::RendererOptions {
        device_pixel_ratio: 1.0,
        resource_override_path: None,
        enable_aa: false,
        enable_msaa: false,
        enable_profiler: false,
        enable_recording: false,
        enable_scrollbars: false,
        debug: false,
        enable_subpixel_aa: false,
        precache_shaders: false,
        renderer_kind: RendererKind::Native,
    };

    let (mut renderer, sender) = webrender::renderer::Renderer::new(opts);
    let api = sender.create_api();

    let notifier = Box::new(Notifier::new(window.create_window_proxy()));
    renderer.set_render_notifier(notifier);

    let pipeline_id = PipelineId(0, 0);

    let mut auxiliary_lists_builder = AuxiliaryListsBuilder::new();
    let mut builder = webrender_traits::DisplayListBuilder::new();

    let bounds = euclid::Rect::new(Point2D::new(0.0, 0.0), window_size);
    builder.push_stacking_context(
        webrender_traits::StackingContext::new(webrender_traits::ScrollPolicy::Scrollable,
                                               bounds,
                                               bounds,
                                               0,
                                               &Matrix4D::identity(),
                                               &Matrix4D::identity(),
                                               webrender_traits::MixBlendMode::Normal,
                                               Vec::new(),
                                               &mut auxiliary_lists_builder));

    item.as_display_item(ClipRegion::simple(&bounds), &mut builder);

    let epoch = Epoch(0);
    let root_background_color = ColorF::new(0.3, 0.0, 0.0, 1.0);
    builder.pop_stacking_context();
    api.set_root_display_list(
         root_background_color,
         epoch,
         pipeline_id,
         Size2D::new(width as f32, height as f32),
         builder.finalize(),
         auxiliary_lists_builder.finalize());

    api.set_root_pipeline(pipeline_id);

    for event in window.wait_events() {
        gl::clear(gl::COLOR_BUFFER_BIT);
        renderer.update();

        renderer.render(Size2D::new(width, height));

        window.swap_buffers().ok();

        match event {
            glutin::Event::Closed => break,
            glutin::Event::KeyboardInput(_element_state, scan_code, _virtual_key_code) => {
                if scan_code == 9 {
                    break;
                }
            }
            _ => (),
        }
    }
}

pub trait AsDisplayItem {
    fn as_display_item(&self, clip: ClipRegion, builder: &mut webrender_traits::DisplayListBuilder);
}

pub struct Rect {
    pub origin_x: f32,
    pub origin_y: f32,
    pub size_x: f32,
    pub size_y: f32,
}

impl Rect {
    fn as_euclid_rect(&self) -> euclid::Rect<f32> {
        euclid::Rect::new(Point2D::new(self.origin_x, self.origin_y), Size2D::new(self.size_x, self.size_y))
    }
}


impl AsDisplayItem for Rect {
    fn as_display_item(&self, clip: ClipRegion, builder: &mut webrender_traits::DisplayListBuilder) {
        builder.push_rect(self.as_euclid_rect(), clip, ColorF::new(0.0, 1.0, 0.0, 1.0))
    }
}


