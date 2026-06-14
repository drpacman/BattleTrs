use sdl2::pixels::Color;
use sdl2::rect::Rect;

use super::{draw_text, text_w, Renderer, WINDOW_H, WINDOW_W};

pub fn draw_game_over(
    r: &mut Renderer,
    won: bool,
    score: u32,
    lines: u32,
    winner_name: Option<&str>,
    elo_delta: Option<i32>,
) {
    let cx = WINDOW_W as i32 / 2;
    let cy = WINDOW_H as i32 / 2;

    // Dark overlay
    r.canvas.set_draw_color(Color::RGBA(0, 0, 0, 200));
    let _ = r.canvas.fill_rect(Rect::new(0, 0, WINDOW_W, WINDOW_H));

    let panel_h = if winner_name.is_some() { 390u32 } else { 340u32 };

    // Result panel
    r.canvas.set_draw_color(Color::RGB(20, 10, 40));
    let _ = r.canvas.fill_rect(Rect::new(cx - 220, cy - 195, 440, panel_h));
    r.canvas.set_draw_color(Color::RGB(80, 0, 160));
    let _ = r.canvas.draw_rect(Rect::new(cx - 220, cy - 195, 440, panel_h));

    // Win/Lose text scale=4
    let (result_text, result_color) = if won {
        ("YOU WIN!", Color::RGB(50, 220, 50))
    } else {
        ("GAME OVER", Color::RGB(220, 50, 50))
    };
    draw_text(
        &mut r.canvas,
        result_text,
        cx - text_w(result_text, 4) / 2,
        cy - 170,
        result_color,
        4,
    );

    // Opponent name in network mode
    let mut y = cy - 100;
    if let Some(name) = winner_name {
        let label = if won {
            format!("You beat {name}!")
        } else {
            format!("{name} wins")
        };
        draw_text(&mut r.canvas, &label, cx - text_w(&label, 2) / 2, y,
            Color::RGB(200, 200, 200), 2);
        y += 26;
    }

    // Stats
    let score_str = format!("SCORE: {score}");
    let lines_str = format!("LINES: {lines}");
    draw_text(&mut r.canvas, &score_str, cx - text_w(&score_str, 3) / 2, y, Color::RGB(255, 220, 0), 3);
    y += 30;
    draw_text(&mut r.canvas, &lines_str, cx - text_w(&lines_str, 3) / 2, y, Color::RGB(200, 200, 200), 3);
    y += 40;

    // ELO delta in network mode
    if let Some(delta) = elo_delta {
        let (delta_str, delta_col) = if delta >= 0 {
            (format!("ELO: +{delta}"), Color::RGB(80, 220, 80))
        } else {
            (format!("ELO: {delta}"), Color::RGB(220, 80, 80))
        };
        draw_text(&mut r.canvas, &delta_str, cx - text_w(&delta_str, 3) / 2, y,
            delta_col, 3);
        y += 40;
    }

    // Prompts
    let p1 = "ENTER  -  PLAY AGAIN";
    let p2 = "ESC  -  TITLE SCREEN";
    draw_text(&mut r.canvas, p1, cx - text_w(p1, 2) / 2, y,      Color::RGB(180, 180, 180), 2);
    draw_text(&mut r.canvas, p2, cx - text_w(p2, 2) / 2, y + 25, Color::RGB(180, 180, 180), 2);
}
