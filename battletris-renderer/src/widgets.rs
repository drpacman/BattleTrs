use crate::{Color, DrawContext};
use crate::font::{draw_text, text_w};

pub fn draw_input_field<D: DrawContext>(
    ctx: &mut D,
    x: f64,
    y: f64,
    w: f64,
    text: &str,
    active: bool,
    cursor_visible: bool,
) {
    const H: f64 = 40.0;
    let bg     = if active { Color::rgb(35, 35, 70)    } else { Color::rgb(20, 20, 40)   };
    let border = if active { Color::rgb(100, 100, 220) } else { Color::rgb(60, 60, 100)  };
    ctx.fill_rect(x, y, w, H, bg);
    ctx.stroke_rect(x, y, w, H, border);

    // Truncate from the left so the cursor end stays visible.
    let display = if text.len() > 40 { &text[text.len() - 40..] } else { text };
    draw_text(ctx, display, x + 6.0, y + 10.0, Color::rgb(220, 220, 220), 2.0);

    if active && cursor_visible {
        let cursor_x = x + 6.0 + text_w(display, 2.0);
        ctx.fill_rect(cursor_x, y + 8.0, 2.0, 24.0, Color::rgb(200, 200, 200));
    }
}
