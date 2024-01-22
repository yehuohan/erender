//! Pure software renderer

use eframe::egui;
use magx::*;
use rasterizer::rasterizer::Rasterizer;
use rasterizer::shader::IGlsl;
use scene::scene::Scene;
use std::time::Instant;

struct SoftRenderer {
    scene: Scene,
    rasterizer: Rasterizer,
    draw_color: bool,
    draw_depth: bool,
}

impl SoftRenderer {
    fn new(sz: (u32, u32)) -> Self {
        // Load scene
        let start = Instant::now();
        let mut scene = Scene::new(sz);
        println!("scene load time: {} ms", start.elapsed().as_millis());

        // Update scene with rasterizer
        let start = Instant::now();
        let mut rasterizer = Rasterizer::new(scene.sz);
        scene.update(&mut rasterizer);
        println!("render time: {} ms", start.elapsed().as_millis());

        Self {
            scene,
            rasterizer,
            draw_color: true,
            draw_depth: true,
        }
    }

    fn handle_keys(&mut self, key: &egui::Key, pressed: &bool, modifiers: &egui::Modifiers) {
        let en_cull_back_face = self.rasterizer.glsl_vars().en_cull_back_face;
        let en_wire_frame = self.rasterizer.glsl_vars().en_wire_frame;
        if *pressed && modifiers.is_none() {
            match key {
                egui::Key::E => self.scene.comps.borrow_mut().camera.rotate_x(Angle::Ang(10.0)),
                egui::Key::D => self.scene.comps.borrow_mut().camera.rotate_x(Angle::Ang(-10.0)),
                egui::Key::S => self.scene.comps.borrow_mut().camera.rotate_y(Angle::Ang(10.0)),
                egui::Key::F => self.scene.comps.borrow_mut().camera.rotate_y(Angle::Ang(-10.0)),
                egui::Key::W => self.scene.comps.borrow_mut().camera.rotate_z(Angle::Ang(10.0)),
                egui::Key::R => self.scene.comps.borrow_mut().camera.rotate_z(Angle::Ang(-10.0)),
                egui::Key::G => self.scene.comps.borrow_mut().camera.move_forward(1.0),
                egui::Key::A => self.scene.comps.borrow_mut().camera.move_forward(-1.0),
                egui::Key::Z => self.rasterizer.enable_wire_frame(!en_wire_frame),
                egui::Key::X => self.rasterizer.enable_cull_face(!en_cull_back_face),
                egui::Key::C => self.draw_color = !self.draw_color,
                egui::Key::V => self.draw_depth = !self.draw_depth,
                _ => {}
            }
        }
        if !pressed && egui::Key::Escape == *key {
            std::process::exit(0);
        }
    }
}

impl eframe::App for SoftRenderer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle key events
        ctx.input(|stt| {
            for evt in &stt.events {
                if let egui::Event::Key {
                    key, pressed, modifiers, ..
                } = evt
                {
                    self.handle_keys(key, pressed, modifiers);
                }
            }
        });

        // Controller panel
        egui::SidePanel::left(egui::Id::new("Controller"))
            .show_separator_line(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("Rotate[e,d,s,f,w,r]"));
                ui.label(egui::RichText::new("Move[a,g]"));
                ui.checkbox(&mut self.rasterizer.wire_frame(), egui::RichText::new("Wire[z]"));
                ui.checkbox(&mut self.rasterizer.cull_face(), egui::RichText::new("Cull[x]"));
                ui.checkbox(&mut self.draw_color, egui::RichText::new("Color[c]"));
                ui.checkbox(&mut self.draw_depth, egui::RichText::new("Depth[v]"));
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let wid = self.rasterizer.sz.0 as usize;
            let hei = self.rasterizer.sz.1 as usize;
            let size = [wid, hei];
            ui.horizontal(|ui| {
                // Draw color
                if self.draw_color {
                    let buf = self.rasterizer.get_color();
                    let mut pixels = vec![egui::Color32::BLACK; wid * hei];
                    for x in 0..wid {
                        for y in 0..hei {
                            let p = buf[x + (hei - (y + 1)) * wid];
                            pixels[x + y * wid] = egui::Color32::from_rgba_premultiplied(p[0], p[1], p[2], p[3]);
                        }
                    }
                    let image = egui::ColorImage { size, pixels };
                    let texture = ui.ctx().load_texture("Color", image, Default::default());
                    ui.image((texture.id(), texture.size_vec2()));
                }
                // Draw depth
                if self.draw_depth {
                    let buf = self.rasterizer.get_depth();
                    let mut pixels = vec![egui::Color32::BLACK; wid * hei];
                    for x in 0..wid {
                        for y in 0..hei {
                            let mut p = buf[x + (hei - (y + 1)) * wid];
                            // Make depth more visual
                            if p < 1.0 {
                                p = p.ln_1p();
                            }
                            pixels[x + y * wid] = egui::Color32::from_gray((p * 255.0) as u8);
                        }
                    }
                    let image = egui::ColorImage { size, pixels };
                    let texture = ui.ctx().load_texture("Depth", image, Default::default());
                    ui.image((texture.id(), texture.size_vec2()));
                }
            });
        });

        self.scene.update(&mut self.rasterizer);
    }
}

pub fn run(sz: (u32, u32)) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([sz.0 as f32 * 2.0 + 195.0, sz.1 as f32 + 15.0])
            .with_resizable(false),
        ..Default::default()
    };

    let creator = Box::new(move |cc: &eframe::CreationContext<'_>| -> Box<dyn eframe::App> {
        let mut fonts = egui::FontDefinitions::empty();
        fonts.font_data.insert(
            "FantasqueSansMono".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/FantasqueSansMono-Regular.ttf")),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .push("FantasqueSansMono".to_owned());
        fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .push("FantasqueSansMono".to_owned());
        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.style_mut(|style| {
            style.spacing.item_spacing = egui::vec2(10.0, 10.0);
            if let Some(fondid) = style.text_styles.get_mut(&egui::TextStyle::Body) {
                *fondid = egui::FontId::new(16.0, egui::FontFamily::Monospace);
            }
            if let Some(fondid) = style.text_styles.get_mut(&egui::TextStyle::Button) {
                *fondid = egui::FontId::new(16.0, egui::FontFamily::Monospace);
            }
        });

        Box::new(SoftRenderer::new(sz))
    });

    eframe::run_native("Soft Render", options, creator)
}
