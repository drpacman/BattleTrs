use battletris_renderer::{Color, DrawContext};
use battletris_renderer::font::{draw_text, text_w};
use battletris_renderer::layout::{WINDOW_H, WINDOW_W};

use super::Renderer;

pub fn draw_title(r: &mut Renderer) {
    let mut ctx = r.backend();
    let cx = WINDOW_W / 2.0;

    ctx.fill_rect(0.0, 0.0, WINDOW_W, WINDOW_H, Color::rgb(20, 0, 40));
    ctx.fill_rect(0.0, 200.0, WINDOW_W, 130.0, Color::rgb(50, 0, 100));

    let title = "BATTLETRIS";
    draw_text(&mut ctx, title, cx - text_w(title, 5.0) / 2.0, 225.0, Color::rgb(255, 220, 0), 5.0);

    let sub = "RUST PORT  V1.0";
    draw_text(&mut ctx, sub, cx - text_w(sub, 2.0) / 2.0, 290.0, Color::rgb(200, 200, 200), 2.0);

    ctx.fill_rect(cx - 200.0, 360.0, 400.0, 2.0, Color::rgb(80, 0, 160));

    let s1 = "ENTER  -  PLAY VS COMPUTER";
    draw_text(&mut ctx, s1, cx - text_w(s1, 2.0) / 2.0, 380.0, Color::WHITE, 2.0);

    let s2 = "S  -  SOLO PRACTICE";
    draw_text(&mut ctx, s2, cx - text_w(s2, 2.0) / 2.0, 415.0, Color::rgb(200, 255, 200), 2.0);

    let s3 = "N  -  NETWORK GAME";
    draw_text(&mut ctx, s3, cx - text_w(s3, 2.0) / 2.0, 450.0, Color::WHITE, 2.0);

    ctx.fill_rect(0.0, 555.0, WINDOW_W, 190.0, Color::rgb(40, 0, 60));

    let c1 = "ARROW KEYS: MOVE / ROTATE CW";
    draw_text(&mut ctx, c1, cx - text_w(c1, 2.0) / 2.0, 575.0, Color::rgb(160, 160, 160), 2.0);

    let c2 = "Z: ROTATE CCW   SPACE: HARD DROP   DOWN: SOFT DROP";
    draw_text(&mut ctx, c2, cx - text_w(c2, 1.0) / 2.0, 608.0, Color::rgb(160, 160, 160), 1.0);

    let c3 = "P: PAUSE   ESC: QUIT   B: OPEN SHOP (SOLO)";
    draw_text(&mut ctx, c3, cx - text_w(c3, 1.0) / 2.0, 622.0, Color::rgb(160, 160, 160), 1.0);

    let c4 = "1-9/0: LAUNCH WEAPON";
    draw_text(&mut ctx, c4, cx - text_w(c4, 1.0) / 2.0, 636.0, Color::rgb(140, 200, 140), 1.0);

    let cred = "ORIGINAL BATTLETRIS (1994)  BROWN UNIV CS32";
    draw_text(&mut ctx, cred, cx - text_w(cred, 1.0) / 2.0, 760.0, Color::rgb(80, 80, 80), 1.0);
}
