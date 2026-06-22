use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

/// Raw key code string used to signal a keyup event.
const UP_PREFIX: &str = "UP:";

pub struct InputHandler {
    queue: Rc<RefCell<VecDeque<String>>>,
    _on_keydown: Closure<dyn FnMut(KeyboardEvent)>,
    _on_keyup: Closure<dyn FnMut(KeyboardEvent)>,
}

impl InputHandler {
    pub fn new() -> Self {
        let queue = Rc::new(RefCell::new(VecDeque::new()));

        let q_down = queue.clone();
        let on_keydown = Closure::wrap(Box::new(move |e: KeyboardEvent| {
            let code = e.code();
            if should_prevent_default(&code) {
                e.prevent_default();
            }
            q_down.borrow_mut().push_back(code);
        }) as Box<dyn FnMut(KeyboardEvent)>);

        let q_up = queue.clone();
        let on_keyup = Closure::wrap(Box::new(move |e: KeyboardEvent| {
            let code = e.code();
            // Only track keyup for keys that need release signals
            if code == "ArrowDown" {
                q_up.borrow_mut().push_back(format!("{UP_PREFIX}{code}"));
            }
        }) as Box<dyn FnMut(KeyboardEvent)>);

        let window = web_sys::window().unwrap();
        let target: &web_sys::EventTarget = window.as_ref();
        target
            .add_event_listener_with_callback("keydown", on_keydown.as_ref().unchecked_ref())
            .unwrap();
        target
            .add_event_listener_with_callback("keyup", on_keyup.as_ref().unchecked_ref())
            .unwrap();

        Self {
            queue,
            _on_keydown: on_keydown,
            _on_keyup: on_keyup,
        }
    }

    /// Drain all queued key events since the last tick. Returns raw code strings.
    pub fn drain(&self) -> Vec<String> {
        self.queue.borrow_mut().drain(..).collect()
    }
}

/// Keys for which browser defaults (scroll, etc.) should be suppressed.
fn should_prevent_default(code: &str) -> bool {
    matches!(
        code,
        "ArrowLeft"
            | "ArrowRight"
            | "ArrowUp"
            | "ArrowDown"
            | "Space"
            | "PageUp"
            | "PageDown"
    )
}
