//! Pure software renderer

use eframe::egui;
use image;
use magx::*;
use rasterizer::rasterizer::Rasterizer;
use rasterizer::shader::IGlsl;
use scene::scene::Scene;
use std::collections::HashMap;
use std::time::Instant;

struct SoftRenderer {
    scene: Scene,
    rasterizer: Rasterizer,
    draw_color: bool,
    draw_depth: bool,
    /// Request redraw scene
    redraw: bool,
    /// Meshes to draw on scene
    meshes: HashMap<&'static str, bool>,
}

impl SoftRenderer {
    fn new(sz: (u32, u32)) -> Self {
        // Load scene
        let start = Instant::now();
        let mut scene = Scene::new(sz);
        println!("Scene load time: {} ms", start.elapsed().as_millis());

        // Create rasterizer
        let mut rasterizer = Rasterizer::new(scene.sz);

        // Create meshes list to draw
        let mut meshes: HashMap<&'static str, bool> = HashMap::new();
        for name in scene.get_meshes() {
            meshes.insert(name, false);
        }
        if let Some(name) = meshes.get_mut("frustum") {
            *name = true;
        }
        if let Some(name) = meshes.get_mut("floor") {
            *name = true;
        }

        // Update scene with rasterizer
        let start = Instant::now();
        scene.update(&mut rasterizer, &meshes);
        println!("Render time: {} ms", start.elapsed().as_millis());

        Self {
            scene,
            rasterizer,
            draw_color: true,
            draw_depth: true,
            redraw: true,
            meshes,
        }
    }

    fn save(&self) {
        let wid = self.rasterizer.sz.0;
        let hei = self.rasterizer.sz.1;
        let mut img = image::ImageBuffer::from_pixel(wid, hei, image::Rgba([0, 0, 0, 0]));
        {
            let buf = self.rasterizer.get_color();
            for x in 0..wid {
                for y in 0..hei {
                    img.put_pixel(x, hei - (y + 1), image::Rgba(buf[(x + y * wid) as usize]));
                }
            }
            img.save("soft_renderer_color.tga").unwrap();
        }
        {
            let buf = self.rasterizer.get_depth();
            for x in 0..wid {
                for y in 0..hei {
                    let c = (buf[(x + y * wid) as usize] * 255.0) as u8;
                    img.put_pixel(x, hei - (y + 1), image::Rgba([c, c, c, 255]));
                }
            }
            img.save("soft_renderer_depth.tga").unwrap();
        }
    }

    fn handle_keys(&mut self, key: &egui::Key, pressed: &bool, modifiers: &egui::Modifiers) {
        if *pressed && modifiers.is_none() {
            self.redraw = true;
            match key {
                egui::Key::E => self.scene.comps.borrow_mut().camera.rotate_x(Angle::Ang(10.0)),
                egui::Key::D => self.scene.comps.borrow_mut().camera.rotate_x(Angle::Ang(-10.0)),
                egui::Key::S => self.scene.comps.borrow_mut().camera.rotate_y(Angle::Ang(10.0)),
                egui::Key::F => self.scene.comps.borrow_mut().camera.rotate_y(Angle::Ang(-10.0)),
                egui::Key::W => self.scene.comps.borrow_mut().camera.rotate_z(Angle::Ang(10.0)),
                egui::Key::R => self.scene.comps.borrow_mut().camera.rotate_z(Angle::Ang(-10.0)),
                egui::Key::G => self.scene.comps.borrow_mut().camera.move_forward(1.0),
                egui::Key::A => self.scene.comps.borrow_mut().camera.move_forward(-1.0),
                egui::Key::Z => *self.rasterizer.wire_frame() = !*self.rasterizer.wire_frame(),
                egui::Key::X => *self.rasterizer.cull_face() = !*self.rasterizer.cull_face(),
                egui::Key::C => self.draw_color = !self.draw_color,
                egui::Key::V => self.draw_depth = !self.draw_depth,
                _ => self.redraw = false,
            }
        }
        if *pressed && egui::Key::S == *key && modifiers.command_only() {
            self.save();
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
                ui.label("Rotate: {e,d,s,f,w,r}");
                ui.label("Move: {a,g}");
                if ui.checkbox(&mut self.rasterizer.wire_frame(), "Wire[z]").changed() {
                    self.redraw = true;
                }
                if ui.checkbox(&mut self.rasterizer.cull_face(), "Cull[x]").changed() {
                    self.redraw = true;
                }
                ui.checkbox(&mut self.draw_color, "Color[c]");
                ui.checkbox(&mut self.draw_depth, "Depth[v]");
                ui.label("Meshes:");
                for name in self.scene.get_meshes() {
                    if let Some(mut draw) = self.meshes.get_mut(name) {
                        ui.horizontal(|ui| {
                            ui.label("  ");
                            if ui.checkbox(&mut draw, name).changed() {
                                self.redraw = true;
                            }
                        });
                    }
                }
                if ui.button("Save[^s]").clicked() {
                    SoftRenderer::save(self);
                };
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

        if self.redraw {
            self.scene.update(&mut self.rasterizer, &self.meshes);
        }
        self.redraw = false;
    }
}

pub fn run(sz: (u32, u32)) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([sz.0 as f32 * 2.0 + 215.0, sz.1 as f32 + 15.0])
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
            style.spacing.item_spacing = egui::vec2(10.0, 5.0);
            style.spacing.button_padding.x = 15.0;
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
