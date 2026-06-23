mod app;
mod input;
mod renderer;
mod transport;

use std::cell::RefCell;

use wasm_bindgen::prelude::*;

use app::WasmApp;

thread_local! {
    static APP:  RefCell<Option<WasmApp>>                  = RefCell::new(None);
    static TICK: RefCell<Option<Closure<dyn FnMut(f64)>>>  = RefCell::new(None);
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();

    match WasmApp::new() {
        Ok(app) => {
            APP.with(|a| *a.borrow_mut() = Some(app));
        }
        Err(e) => {
            let _ = web_sys::window()
                .unwrap()
                .alert_with_message(&format!("Init failed: {:?}", e));
            return;
        }
    }

    TICK.with(|t| {
        *t.borrow_mut() = Some(Closure::wrap(Box::new(|ts: f64| {
            APP.with(|a| {
                if let Some(ref mut app) = *a.borrow_mut() {
                    app.tick(ts);
                }
            });
            schedule_next_frame();
        }) as Box<dyn FnMut(f64)>));
    });

    schedule_next_frame();
}

fn schedule_next_frame() {
    TICK.with(|t| {
        if let Some(ref cb) = *t.borrow() {
            let _ = web_sys::window()
                .unwrap()
                .request_animation_frame(cb.as_ref().unchecked_ref());
        }
    });
}
