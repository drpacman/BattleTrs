use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use js_sys::{ArrayBuffer, Uint8Array};
use web_sys::{BinaryType, CloseEvent, Event, MessageEvent, WebSocket};

use battletris_engine::protocol::{self, GameMessage};

pub struct WsTransport {
    ws: WebSocket,
    incoming: Rc<RefCell<VecDeque<GameMessage>>>,
    connected: Rc<Cell<bool>>,
    disconnected: Rc<Cell<bool>>,
    _on_message: Closure<dyn FnMut(MessageEvent)>,
    _on_open: Closure<dyn FnMut(Event)>,
    _on_close: Closure<dyn FnMut(CloseEvent)>,
    _on_error: Closure<dyn FnMut(Event)>,
}

impl WsTransport {
    pub fn connect(url: &str, player_name: &str) -> Result<Self, wasm_bindgen::JsValue> {
        let ws = WebSocket::new(url)?;
        ws.set_binary_type(BinaryType::Arraybuffer);

        let incoming = Rc::new(RefCell::new(VecDeque::<GameMessage>::new()));
        let connected = Rc::new(Cell::new(false));
        let disconnected = Rc::new(Cell::new(false));

        // onmessage: decode binary frame → push to queue
        let incoming_c = incoming.clone();
        let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(buf) = e.data().dyn_into::<ArrayBuffer>() {
                let bytes = Uint8Array::new(&buf).to_vec();
                if let Ok(msg) = protocol::decode_raw(&bytes) {
                    incoming_c.borrow_mut().push_back(msg);
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

        // onopen: mark connected and send Hello
        let ws_c = ws.clone();
        let name = player_name.to_string();
        let connected_c = connected.clone();
        let on_open = Closure::wrap(Box::new(move |_: Event| {
            connected_c.set(true);
            let hello = GameMessage::Hello { name: name.clone() };
            if let Ok(bytes) = protocol::encode_raw(&hello) {
                let arr = Uint8Array::from(bytes.as_slice());
                let _ = ws_c.send_with_array_buffer(&arr.buffer());
            }
        }) as Box<dyn FnMut(Event)>);
        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));

        // onclose: mark disconnected
        let disconnected_c = disconnected.clone();
        let connected_c2 = connected.clone();
        let on_close = Closure::wrap(Box::new(move |_: CloseEvent| {
            connected_c2.set(false);
            disconnected_c.set(true);
        }) as Box<dyn FnMut(CloseEvent)>);
        ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));

        // onerror: mark disconnected
        let disconnected_e = disconnected.clone();
        let connected_e = connected.clone();
        let on_error = Closure::wrap(Box::new(move |_: Event| {
            connected_e.set(false);
            disconnected_e.set(true);
        }) as Box<dyn FnMut(Event)>);
        ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));

        Ok(Self {
            ws,
            incoming,
            connected,
            disconnected,
            _on_message: on_message,
            _on_open: on_open,
            _on_close: on_close,
            _on_error: on_error,
        })
    }

    pub fn drain_incoming(&self) -> Vec<GameMessage> {
        self.incoming.borrow_mut().drain(..).collect()
    }

    pub fn send(&self, msg: &GameMessage) {
        if !self.connected.get() {
            return;
        }
        if let Ok(bytes) = protocol::encode_raw(msg) {
            let arr = Uint8Array::from(bytes.as_slice());
            let _ = self.ws.send_with_array_buffer(&arr.buffer());
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected.get()
    }

    pub fn is_disconnected(&self) -> bool {
        self.disconnected.get()
    }
}
