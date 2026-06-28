pub mod bazaar;
pub mod color;
pub mod context;
pub mod font;
pub mod game_over;
pub mod layout;
pub mod playing;
pub mod primitives;

pub use color::Color;
pub use context::DrawContext;

pub const GIMP_PNG: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/gimp.png"));
