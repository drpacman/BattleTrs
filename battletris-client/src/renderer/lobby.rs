use battletris_renderer::screens::{
    draw_connection_screen as renderer_draw_connection_screen,
    draw_connecting as renderer_draw_connecting,
    draw_waiting as renderer_draw_waiting,
};

use super::Renderer;

pub fn draw_connection_screen(
    r: &mut Renderer,
    addr_buf: &str,
    name_buf: &str,
    active_field: usize,
    cursor_visible: bool,
    error: Option<&str>,
) {
    renderer_draw_connection_screen(
        &mut r.backend(),
        Some((addr_buf, active_field == 0)),
        name_buf,
        active_field == 1,
        cursor_visible,
        error,
    );
}

pub fn draw_connecting_screen(r: &mut Renderer, addr: &str) {
    renderer_draw_connecting(&mut r.backend(), addr);
}

pub fn draw_waiting_screen(r: &mut Renderer, player_name: &str) {
    renderer_draw_waiting(&mut r.backend(), player_name);
}
