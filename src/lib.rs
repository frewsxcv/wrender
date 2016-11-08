extern crate app_units;
extern crate webrender;
extern crate glutin;
extern crate gleam;
extern crate webrender_traits;
extern crate euclid;

use euclid::{Size2D, Point2D, Rect, Matrix4D};
use gleam::gl;
use std::path::PathBuf;
use webrender_traits::{PipelineId, StackingContextId, DisplayListId};
use webrender_traits::{AuxiliaryListsBuilder, Epoch, ColorF};
use webrender_traits::{RendererKind};
use std::fs::File;
use std::io::Read;
use std::env;

struct Notifier {
    window_proxy: glutin::WindowProxy,
}

impl Notifier {
    fn new(window_proxy: glutin::WindowProxy) -> Notifier {
        Notifier {
            window_proxy: window_proxy,
        }
    }
}

pub struct WebRenderFrameBuilder {
    pub stacking_contexts: Vec<(StackingContextId, webrender_traits::StackingContext)>,
    pub display_lists: Vec<(DisplayListId, webrender_traits::BuiltDisplayList)>,
    pub auxiliary_lists_builder: AuxiliaryListsBuilder,
    pub root_pipeline_id: PipelineId,
    pub next_scroll_layer_id: usize,
}

impl WebRenderFrameBuilder {
    pub fn new(root_pipeline_id: PipelineId) -> WebRenderFrameBuilder {
        WebRenderFrameBuilder {
            stacking_contexts: vec![],
            display_lists: vec![],
            auxiliary_lists_builder: AuxiliaryListsBuilder::new(),
            root_pipeline_id: root_pipeline_id,
            next_scroll_layer_id: 0,
        }
    }

    pub fn add_stacking_context(&mut self,
                                api: &mut webrender_traits::RenderApi,
                                pipeline_id: PipelineId,
                                stacking_context: webrender_traits::StackingContext)
                                -> StackingContextId {
        assert!(pipeline_id == self.root_pipeline_id);
        let id = api.next_stacking_context_id();
        self.stacking_contexts.push((id, stacking_context));
        id
    }

    pub fn add_display_list(&mut self,
                            api: &mut webrender_traits::RenderApi,
                            display_list: webrender_traits::BuiltDisplayList,
                            stacking_context: &mut webrender_traits::StackingContext)
                            -> DisplayListId {
        let id = api.next_display_list_id();
        stacking_context.display_lists.push(id);
        self.display_lists.push((id, display_list));
        id
    }

    pub fn next_scroll_layer_id(&mut self) -> webrender_traits::ScrollLayerId {
        let scroll_layer_id = webrender_traits::ServoScrollRootId(self.next_scroll_layer_id);
        self.next_scroll_layer_id += 1;
        webrender_traits::ScrollLayerId::new(self.root_pipeline_id, 0, scroll_layer_id)
    }

}

impl webrender_traits::RenderNotifier for Notifier {
    fn new_frame_ready(&mut self) {
        self.window_proxy.wakeup_event_loop();
    }

    fn new_scroll_frame_ready(&mut self, _composite_needed: bool) {
        self.window_proxy.wakeup_event_loop();
    }

    fn pipeline_size_changed(&mut self,
                             _: PipelineId,
                             _: Option<Size2D<f32>>) {
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("{} <shader path>", args[0]);
        return;
    }

    let res_path = &args[1];

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

    let opts = webrender::RendererOptions {
        device_pixel_ratio: 1.0,
        resource_path: PathBuf::from(res_path),
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
    let mut api = sender.create_api();

    let notifier = Box::new(Notifier::new(window.create_window_proxy()));
    renderer.set_render_notifier(notifier);

    let pipeline_id = PipelineId(0, 0);

    let mut frame_builder = WebRenderFrameBuilder::new(pipeline_id);
    let root_scroll_layer_id = frame_builder.next_scroll_layer_id();

    let bounds = Rect::new(Point2D::new(0.0, 0.0),
                           Size2D::new(width as f32, height as f32));
    let mut sc =
        webrender_traits::StackingContext::new(Some(root_scroll_layer_id),
                                               webrender_traits::ScrollPolicy::Scrollable,
                                               bounds,
                                               bounds,
                                               0,
                                               &Matrix4D::identity(),
                                               &Matrix4D::identity(),
                                               true,
                                               webrender_traits::MixBlendMode::Normal,
                                               Vec::new(),
                                               &mut frame_builder.auxiliary_lists_builder);

    let builder = webrender_traits::DisplayListBuilder::new();


    frame_builder.add_display_list(&mut api, builder.finalize(), &mut sc);
    //let sc_id = frame_builder.add_stacking_context(&mut api, pipeline_id, sc);

    /*
    api.set_root_stacking_context(sc_id,
                                  ColorF::new(1., 0.0, 0.0, 1.0),
                                  Epoch(0),
                                  pipeline_id,
                                  Size2D::new(width as f32, height as f32),
                                  frame_builder.stacking_contexts,
                                  frame_builder.display_lists,
                                  frame_builder.auxiliary_lists_builder
                                               .finalize());
   */

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
            _ => ()
        }
    }
}

