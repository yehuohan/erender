//! 渲染显示器

use std::cell::RefCell;
use image;
use piston_window::{
    PistonWindow, WindowSettings,
    Texture, TextureSettings,
    Events, EventSettings, EventLoop,
    Button, PressEvent, Key,
    RenderEvent,
    text,
    Transformed,
};


/// viewer支持的buffer类型
pub enum EBuffer<'a> {
    Color(&'a Vec<[u8; 4]>),
    Depth(&'a Vec<f32>),
}

pub struct Viewer {
    img: RefCell<image::RgbaImage>,
    txt: RefCell<String>,
}

impl Viewer {
    pub fn new(sz: (u32, u32)) -> Self {
        Self {
            img: RefCell::new(image::ImageBuffer::from_pixel(sz.0, sz.1, image::Rgba([0, 0, 0, 0]))),
            txt: RefCell::new(String::new()),
        }
    }

    /// 显示渲染buffer
    pub fn swap(&self, sz: (u32, u32), buffer: EBuffer) {
        let (w, h) = sz;

        let mut img = self.img.borrow_mut();
        let sw = img.width();
        let sh = img.height();

        match buffer {
            EBuffer::Color(ref b) => {
                for x in 0 .. w.min(sw) {
                    for y in 0 .. h.min(sh) {
                        img.put_pixel(x, sh - (y + 1), image::Rgba(b[(x + y * w) as usize]));
                    }
                }
            },
            EBuffer::Depth(ref b) => {
                for x in 0 .. w.min(sw) {
                    for y in 0 .. h.min(sh) {
                        let c = (b[(x + y * w) as usize] * 255.0) as u8;
                        img.put_pixel(x, sh - (y + 1), image::Rgba([c, c, c, 255]));
                    }
                }
            },
        }
    }

    /// 显示文本
    pub fn text(&self, text: &str) {
        let mut txt = self.txt.borrow_mut();
        txt.clear();
        txt.push_str(text);
    }

    /// 保存图片
    pub fn save(&self, filename: Option<&String>) {
        let f = if let Some(f) = filename { f } else { "render.tga" };
        self.img.borrow().save(f).unwrap();
    }

    /// 显示图片（不处理文本）
    pub fn disp(&self) {
        let mut win: PistonWindow = WindowSettings::new(
            "render", [self.img.borrow().width(), self.img.borrow().height()])
                .srgb(false).resizable(false)
                .transparent(true)
                .exit_on_esc(true)
                .build().unwrap();
        let tex = Texture::from_image(
            &mut win.create_texture_context(),
            &*self.img.borrow(),
            &TextureSettings::new()
        ).unwrap();

        while let Some(event) = win.next() {
            win.draw_2d(&event, |ctx, g, _dev| {
                piston_window::image(&tex, ctx.transform, g);
            });
        }
    }

    /// 显示图片和文本
    pub fn run<F: FnMut(&Viewer, char)>(&self, f: &mut F) {
        let mut win: PistonWindow = WindowSettings::new(
            "render", [self.img.borrow().width(), self.img.borrow().height()])
                .srgb(false).resizable(false)
                .transparent(true)
                .exit_on_esc(true)
                .build().unwrap();
        let mut glyphs = win.load_font("assets/fonts/FantasqueSansMono-Regular.ttf").unwrap();
        let font_size = 13;
        let mut events = Events::new(EventSettings::new().lazy(true));

        while let Some(e) = events.next(&mut win) {
            if let Some(_) = e.render_args() {
                // render事件
                let tex = Texture::from_image(
                    &mut win.create_texture_context(),
                    &*self.img.borrow(),
                    &TextureSettings::new()
                ).unwrap();
                win.draw_2d(&e, |ctx, g, dev| {
                    piston_window::image(&tex, ctx.transform, g);

                    text::Text::new_color([0.0, 0.0, 0.0, 1.0], font_size).draw(
                        &self.txt.borrow(), &mut glyphs,
                        &ctx.draw_state, ctx.transform.trans(5.0, 5.0 + (font_size as f64)), g
                    ).unwrap();
                    glyphs.factory.encoder.flush(dev);
                });
            }

            if let Some(Button::Keyboard(key)) = e.press_args() {
                // key press事件
                if Key::A <= key && key <= Key::Z {
                    println!("Key '{}' pressed!", key as u8 as char);
                    f(&self, key as u8 as char);
                }
            }
        }
    }
}
