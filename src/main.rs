use std::time::Instant;
use magx::*;
use scene::scene::Scene;
use rasterizer::{
    rasterizer::Rasterizer,
    shader::IGlsl,
};
use viewer::{Viewer, EBuffer};


fn main() {
    /* load */
    let start = Instant::now();
    let mut s = Scene::new((400, 400));
    println!("scene load time: {} ms", start.elapsed().as_millis());

    /* render */
    let start = Instant::now();
    let mut r = Rasterizer::new(s.sz);
    s.update(&mut r);
    println!("render time: {} ms", start.elapsed().as_millis());

    /* display */
    let u = Viewer::new(s.sz);
    u.swap(r.sz, EBuffer::Color(r.get_color()));
    u.save(None);

    let mut buf_color = true;
    u.run(&mut |v: &Viewer, key: char| {
        let mut redraw = true;
        if key == 'e' {
            s.comps.borrow_mut().camera.rotate_x(Angle::Ang(10.0));
        } else if key == 'd' {
            s.comps.borrow_mut().camera.rotate_x(Angle::Ang(-10.0));
        } else if key == 's' {
            s.comps.borrow_mut().camera.rotate_y(Angle::Ang(10.0));
        } else if key == 'f' {
            s.comps.borrow_mut().camera.rotate_y(Angle::Ang(-10.0));
        } else if key == 'w' {
            s.comps.borrow_mut().camera.rotate_z(Angle::Ang(10.0));
        } else if key == 'r' {
            s.comps.borrow_mut().camera.rotate_z(Angle::Ang(-10.0));
        } else if key == 'g' {
            s.comps.borrow_mut().camera.move_forward(1.0);
        } else if key == 'a' {
            s.comps.borrow_mut().camera.move_forward(-1.0);
        } else if key == 'x' {
            r.enable_cull_face(!r.glsl_vars().en_cull_back_face);
        } else if key == 'c' {
            r.enable_wire_frame(!r.glsl_vars().en_wire_frame);
        } else if key == 'v' {
            buf_color = !buf_color;
            redraw = false;
        }

        if redraw {
            s.update(&mut r);
        }
        v.swap(r.sz, if buf_color {
                EBuffer::Color(r.get_color())
            } else {
                EBuffer::Depth(r.get_depth())
            });
        u.text(&format!("Cull: {}, Wire: {}, Buffer: {}",
                r.glsl_vars().en_cull_back_face,
                r.glsl_vars().en_wire_frame,
                if buf_color { "color" } else { "depth" }));
    });
}
