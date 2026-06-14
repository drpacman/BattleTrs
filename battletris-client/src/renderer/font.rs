use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

// 5×7 pixel bitmap font covering printable ASCII 32–96.
// Index = char_code - 32.  Each glyph is 7 rows; each row byte uses the 5
// low bits: bit 4 = leftmost column, bit 0 = rightmost column.
const GLYPH_W: i32 = 5;

#[rustfmt::skip]
const FONT: &[[u8; 7]] = &[
    [0,  0,  0,  0,  0,  0,  0 ], // 32 ' '
    [4,  4,  4,  4,  0,  4,  0 ], // 33 '!'
    [10, 10, 0,  0,  0,  0,  0 ], // 34 '"'
    [10, 31, 10, 31, 10, 0,  0 ], // 35 '#'
    [4,  14, 20, 14, 5,  14, 4 ], // 36 '$'
    [17, 9,  2,  4,  18, 9,  17], // 37 '%'
    [12, 18, 12, 10, 17, 18, 13], // 38 '&'
    [12, 4,  8,  0,  0,  0,  0 ], // 39 '\''
    [2,  4,  8,  8,  8,  4,  2 ], // 40 '('
    [8,  4,  2,  2,  2,  4,  8 ], // 41 ')'
    [0,  10, 4,  31, 4,  10, 0 ], // 42 '*'
    [0,  4,  4,  31, 4,  4,  0 ], // 43 '+'
    [0,  0,  0,  0,  6,  4,  8 ], // 44 ','
    [0,  0,  0,  31, 0,  0,  0 ], // 45 '-'
    [0,  0,  0,  0,  0,  12, 12], // 46 '.'
    [1,  1,  2,  4,  8,  16, 16], // 47 '/'
    [14, 17, 17, 17, 17, 17, 14], // 48 '0'
    [4,  12, 4,  4,  4,  4,  14], // 49 '1'
    [14, 17, 1,  6,  8,  16, 31], // 50 '2'
    [14, 17, 1,  6,  1,  17, 14], // 51 '3'
    [2,  6,  10, 18, 31, 2,  2 ], // 52 '4'
    [31, 16, 16, 30, 1,  17, 14], // 53 '5'
    [14, 16, 16, 30, 17, 17, 14], // 54 '6'
    [31, 1,  2,  4,  8,  8,  8 ], // 55 '7'
    [14, 17, 17, 14, 17, 17, 14], // 56 '8'
    [14, 17, 17, 15, 1,  17, 14], // 57 '9'
    [0,  12, 12, 0,  12, 12, 0 ], // 58 ':'
    [0,  12, 12, 0,  12, 4,  8 ], // 59 ';'
    [2,  4,  8,  16, 8,  4,  2 ], // 60 '<'
    [0,  31, 0,  0,  31, 0,  0 ], // 61 '='
    [8,  4,  2,  1,  2,  4,  8 ], // 62 '>'
    [14, 17, 1,  6,  4,  0,  4 ], // 63 '?'
    [14, 17, 1,  13, 21, 21, 14], // 64 '@'
    [4,  10, 17, 31, 17, 17, 17], // 65 'A'
    [30, 17, 17, 30, 17, 17, 30], // 66 'B'
    [14, 17, 16, 16, 16, 17, 14], // 67 'C'
    [30, 17, 17, 17, 17, 17, 30], // 68 'D'
    [31, 16, 16, 30, 16, 16, 31], // 69 'E'
    [31, 16, 16, 30, 16, 16, 16], // 70 'F'
    [14, 17, 16, 23, 17, 17, 14], // 71 'G'
    [17, 17, 17, 31, 17, 17, 17], // 72 'H'
    [14, 4,  4,  4,  4,  4,  14], // 73 'I'
    [7,  2,  2,  2,  18, 18, 12], // 74 'J'
    [17, 18, 20, 24, 20, 18, 17], // 75 'K'
    [16, 16, 16, 16, 16, 16, 31], // 76 'L'
    [17, 27, 21, 17, 17, 17, 17], // 77 'M'
    [17, 25, 21, 19, 17, 17, 17], // 78 'N'
    [14, 17, 17, 17, 17, 17, 14], // 79 'O'
    [30, 17, 17, 30, 16, 16, 16], // 80 'P'
    [14, 17, 17, 17, 21, 19, 15], // 81 'Q'
    [30, 17, 17, 30, 20, 18, 17], // 82 'R'
    [14, 17, 16, 14, 1,  17, 14], // 83 'S'
    [31, 4,  4,  4,  4,  4,  4 ], // 84 'T'
    [17, 17, 17, 17, 17, 17, 14], // 85 'U'
    [17, 17, 17, 17, 10, 10, 4 ], // 86 'V'
    [17, 17, 17, 21, 21, 27, 17], // 87 'W'
    [17, 17, 10, 4,  10, 17, 17], // 88 'X'
    [17, 17, 10, 4,  4,  4,  4 ], // 89 'Y'
    [31, 1,  2,  4,  8,  16, 31], // 90 'Z'
    [14, 8,  8,  8,  8,  8,  14], // 91 '['
    [16, 8,  8,  4,  2,  1,  1 ], // 92 '\\'
    [14, 2,  2,  2,  2,  2,  14], // 93 ']'
    [4,  10, 17, 0,  0,  0,  0 ], // 94 '^'
    [0,  0,  0,  0,  0,  0,  31], // 95 '_'
    [8,  4,  0,  0,  0,  0,  0 ], // 96 '`'
];

/// Pixel width of one rendered character including inter-character gap.
pub fn char_step(scale: u32) -> i32 {
    (GLYPH_W + 1) * scale as i32
}

/// Total pixel width of a string at the given scale.
pub fn text_w(text: &str, scale: u32) -> i32 {
    text.chars().count() as i32 * char_step(scale)
}

/// Draw a string using the embedded 5×7 bitmap font.
/// `scale` is the number of SDL pixels per font pixel (1 = tiny, 2 = small label,
/// 3 = medium value, 5 = large title).
/// Lowercase letters are automatically mapped to uppercase.
pub fn draw_text(
    canvas: &mut Canvas<Window>,
    text: &str,
    x: i32,
    y: i32,
    color: Color,
    scale: u32,
) {
    let s = scale as i32;
    let step = char_step(scale);
    canvas.set_draw_color(color);
    for (ci, ch) in text.chars().enumerate() {
        let c = ch.to_ascii_uppercase() as usize;
        if c < 32 || c > 96 {
            continue;
        }
        let glyph = FONT[c - 32];
        let gx = x + ci as i32 * step;
        for (row, &bits) in glyph.iter().enumerate() {
            for col in 0_i32..GLYPH_W {
                let bit_pos = (GLYPH_W - 1 - col) as u32;
                if bits & (1u8 << bit_pos) != 0 {
                    let px = gx + col * s;
                    let py = y + row as i32 * s;
                    let _ = canvas.fill_rect(Rect::new(px, py, scale, scale));
                }
            }
        }
    }
}
