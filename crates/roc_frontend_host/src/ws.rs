use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::js_sys::Function;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

#[wasm_bindgen]
pub struct ReconnectingWebSocket {
    ws_url: String,
    ws: Option<WebSocket>,
    onmessage_callback: Option<Closure<dyn FnMut(MessageEvent)>>,
}

impl ReconnectingWebSocket {
    pub fn new(ws_url: String) -> Self {
        ReconnectingWebSocket {
            ws_url,
            ws: None,
            onmessage_callback: None,
        }
    }

    pub fn connect(&mut self) -> Result<(), JsValue> {
        let ws = WebSocket::new(&self.ws_url)?;
        // ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let onopen_callback = Closure::wrap(Box::new(move || {
            web_sys::console::log_1(&"WebSocket opened".into());
        }) as Box<dyn FnMut()>);
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        if let Some(onmessage_callback) = &self.onmessage_callback {
            ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        }

        let onerror_callback = Closure::wrap(Box::new(move |error: ErrorEvent| {
            web_sys::console::error_1(&error);
        }) as Box<dyn FnMut(ErrorEvent)>);
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        let onclose_callback = Closure::wrap(Box::new(move || {
            web_sys::console::log_1(&"WebSocket closed".into());
        }) as Box<dyn FnMut()>);

        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();

        self.ws = Some(ws);

        Ok(())
    }

    pub fn set_onmessage<F>(&mut self, callback: F)
    where
        F: Fn(MessageEvent) + 'static,
    {
        let closure = Closure::wrap(Box::new(move |event: MessageEvent| {
            callback(event);
        }) as Box<dyn FnMut(MessageEvent)>);

        if let Some(ws) = &self.ws {
            ws.set_onmessage(Some(closure.as_ref().unchecked_ref()));
        }

        self.onmessage_callback = Some(closure);
    }

    pub fn send_message(&self, message: &str) -> Result<(), JsValue> {
        if let Some(ws) = &self.ws {
            ws.send_with_str(message)?;
        }
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), JsValue> {
        if let Some(ws) = &self.ws {
            ws.close()?;
        }
        self.ws = None;
        Ok(())
    }
}
