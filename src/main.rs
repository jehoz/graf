use app::App;
use macroquad::prelude::*;

use crate::drawing_utils::ColorPalette;

mod app;
mod dag;
mod devices;
mod drawing_utils;
mod midi;
mod session;
mod v2;
mod widgets;

fn window_conf() -> Conf {
    Conf {
        window_title: "GRAF".to_owned(),
        sample_count: 8,
        ..Default::default()
    }
}

#[macroquad::main(window_conf())]
async fn main() {
    let colors = ColorPalette {
        fg_0: Color::from_hex(0xFFFFFF),
        fg_1: Color::from_hex(0xE0E0E0),
        fg_2: Color::from_hex(0xB7B7B7),
        fg_3: Color::from_hex(0x8C8C8C),

        bg_0: Color::from_hex(0x000000),
        bg_1: Color::from_hex(0x111111),
        bg_2: Color::from_hex(0x212121),
        bg_3: Color::from_hex(0x3C3C3C),

        error: Color::from_hex(0xF21B1B),
    };

    let mut app = App::new(colors);

    loop {
        let mut egui_wants_pointer = false;
        egui_macroquad::ui(|ctx| {
            app.ui(ctx);
            if ctx.wants_pointer_input() {
                egui_wants_pointer = true;
            }
        });

        if !egui_wants_pointer {
            app.handle_inputs();
        }
        app.update();
        app.draw();
        egui_macroquad::draw();

        next_frame().await
    }
}
