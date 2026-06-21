use sdl2::pixels::Color;
use sdl2::rect::Rect;

use super::{draw_text, text_w, Renderer, WINDOW_H, WINDOW_W};

pub fn draw_title(r: &mut Renderer) {
    let cx = WINDOW_W as i32 / 2;

    // Background
    r.canvas.set_draw_color(Color::RGB(20, 0, 40));
    let _ = r.canvas.fill_rect(Rect::new(0, 0, WINDOW_W, WINDOW_H));

    // Title accent bar
    r.canvas.set_draw_color(Color::RGB(50, 0, 100));
    let _ = r.canvas.fill_rect(Rect::new(0, 200, WINDOW_W, 130));

    // "BATTLETRIS" scale=5 → 300px wide, 35px tall
    let title = "BATTLETRIS";
    draw_text(&mut r.canvas, title, cx - text_w(title, 5) / 2, 225, Color::RGB(255, 220, 0), 5);

    // Subtitle scale=2
    let sub = "RUST PORT  V1.0";
    draw_text(&mut r.canvas, sub, cx - text_w(sub, 2) / 2, 290, Color::RGB(200, 200, 200), 2);

    // Divider
    r.canvas.set_draw_color(Color::RGB(80, 0, 160));
    let _ = r.canvas.fill_rect(Rect::new(cx - 200, 360, 400, 2));

    // Menu entries scale=2
    let s1 = "ENTER  -  PLAY VS COMPUTER";
    draw_text(&mut r.canvas, s1, cx - text_w(s1, 2) / 2, 380, Color::RGB(255, 255, 255), 2);

    let s2 = "S  -  SOLO PRACTICE";
    draw_text(&mut r.canvas, s2, cx - text_w(s2, 2) / 2, 415, Color::RGB(200, 255, 200), 2);

    let s3 = "N  -  NETWORK GAME";
    draw_text(&mut r.canvas, s3, cx - text_w(s3, 2) / 2, 450, Color::RGB(255, 255, 255), 2);

    // Controls block
    r.canvas.set_draw_color(Color::RGB(40, 0, 60));
    let _ = r.canvas.fill_rect(Rect::new(0, 555, WINDOW_W, 190));

    let c1 = "ARROW KEYS: MOVE / ROTATE CW";
    draw_text(&mut r.canvas, c1, cx - text_w(c1, 2) / 2, 575, Color::RGB(160, 160, 160), 2);

    let c2 = "Z: ROTATE CCW   SPACE: HARD DROP   DOWN: SOFT DROP";
    draw_text(&mut r.canvas, c2, cx - text_w(c2, 1) / 2, 608, Color::RGB(160, 160, 160), 1);

    let c3 = "P: PAUSE   ESC: QUIT   B: OPEN SHOP (SOLO)";
    draw_text(&mut r.canvas, c3, cx - text_w(c3, 1) / 2, 622, Color::RGB(160, 160, 160), 1);

    let c4 = "1-9/0: LAUNCH WEAPON";
    draw_text(&mut r.canvas, c4, cx - text_w(c4, 1) / 2, 636, Color::RGB(140, 200, 140), 1);

    // Credits scale=1
    let cred = "ORIGINAL BATTLETRIS (1994)  BROWN UNIV CS32";
    draw_text(&mut r.canvas, cred, cx - text_w(cred, 1) / 2, 760, Color::RGB(80, 80, 80), 1);
}
