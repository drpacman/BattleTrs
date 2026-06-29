use battletris_renderer::title::{
    draw_difficulty_select as renderer_draw_difficulty_select,
    draw_title as renderer_draw_title,
};

use super::Renderer;

pub fn draw_title(r: &mut Renderer) {
    renderer_draw_title(&mut r.backend());
}

pub fn draw_difficulty_select(r: &mut Renderer, selected: usize) {
    renderer_draw_difficulty_select(&mut r.backend(), selected);
}
