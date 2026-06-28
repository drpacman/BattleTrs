mod app;
mod input;
mod renderer;
mod transport;

use std::cell::RefCell;

use wasm_bindgen::prelude::*;

use app::WasmApp;

thread_local! {
    static APP:       RefCell<Option<WasmApp>>                 = RefCell::new(None);
    static TICK:      RefCell<Option<Closure<dyn FnMut(f64)>>> = RefCell::new(None);
    static HEARTBEAT: RefCell<Option<Closure<dyn FnMut()>>>    = RefCell::new(None);
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

    // rAF loop — runs at ~60fps when the tab is focused.
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

    // Background heartbeat — keeps the engine ticking when the tab is hidden.
    //
    // Chrome throttles rAF to 0fps in hidden tabs, which prevents the background
    // player from processing incoming WebSocket messages (e.g. BazaarOpen).
    // This setInterval fires at 100ms when visible (cheap no-op) and at whatever
    // Chrome allows when hidden (throttled to ~1000ms after 5 s, but still enough
    // to keep game state and network messages in sync for two-tab local testing).
    //
    // Date.now() is used instead of rAF's DOMHighResTimeStamp; the 100 ms cap in
    // app.tick() absorbs the timestamp format discontinuity on focus change.
    HEARTBEAT.with(|h| {
        *h.borrow_mut() = Some(Closure::wrap(Box::new(|| {
            let win = web_sys::window().unwrap();
            let doc = win.document().unwrap();
            if doc.hidden() {
                let ts = js_sys::Date::now();
                APP.with(|a| {
                    if let Some(ref mut app) = *a.borrow_mut() {
                        app.tick(ts);
                    }
                });
            }
        }) as Box<dyn FnMut()>));
    });

    let win = web_sys::window().unwrap();
    HEARTBEAT.with(|h| {
        if let Some(ref cb) = *h.borrow() {
            let _ = win.set_interval_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                100,
            );
        }
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
