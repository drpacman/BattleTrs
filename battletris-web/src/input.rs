use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

const UP_PREFIX: &str = "UP:";

pub struct InputHandler {
    /// `e.code()` values — used for game controls (ArrowLeft, KeyZ, Space, …)
    code_queue: Rc<RefCell<VecDeque<String>>>,
    /// `e.key()` values — used for text input in the lobby ("a", "A", "Backspace", …)
    text_queue: Rc<RefCell<VecDeque<String>>>,
    _on_keydown: Closure<dyn FnMut(KeyboardEvent)>,
    _on_keyup: Closure<dyn FnMut(KeyboardEvent)>,
}

impl InputHandler {
    pub fn new() -> Self {
        let code_queue = Rc::new(RefCell::new(VecDeque::new()));
        let text_queue = Rc::new(RefCell::new(VecDeque::new()));

        let cq = code_queue.clone();
        let tq = text_queue.clone();
        let on_keydown = Closure::wrap(Box::new(move |e: KeyboardEvent| {
            let code = e.code();
            if should_prevent_default(&code) {
                e.prevent_default();
            }
            cq.borrow_mut().push_back(code);
            tq.borrow_mut().push_back(e.key());
        }) as Box<dyn FnMut(KeyboardEvent)>);

        let cq_up = code_queue.clone();
        let on_keyup = Closure::wrap(Box::new(move |e: KeyboardEvent| {
            if e.code() == "ArrowDown" {
                cq_up.borrow_mut().push_back(format!("{UP_PREFIX}ArrowDown"));
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
            code_queue,
            text_queue,
            _on_keydown: on_keydown,
            _on_keyup: on_keyup,
        }
    }

    /// Drain raw key-code events for game input mapping.
    pub fn drain(&self) -> Vec<String> {
        self.code_queue.borrow_mut().drain(..).collect()
    }

    /// Drain key values for text / lobby input ("a", "A", "1", "Backspace", …).
    pub fn drain_text(&self) -> Vec<String> {
        self.text_queue.borrow_mut().drain(..).collect()
    }
}

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
            | "Tab"
            | "Backspace"
    )
}
