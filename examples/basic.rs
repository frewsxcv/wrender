extern crate wrender;

fn main() {
    let rect = wrender::Rect {
        origin_x: 100.,
        origin_y: 100.,
        size_x: 100.,
        size_y: 100.,
    };

    wrender::run(rect);
}
