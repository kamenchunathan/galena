mod roc;
mod ws;

use std::cell::RefCell;
use std::sync::{Arc, LazyLock, Mutex};

use roc::Model;
use roc::{frontend_decode_to_frontend_msg, UpdateResult};
use roc_std::RocBox;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::js_sys::Array;
use web_sys::WebSocket;
use web_sys::{self, console};
use web_sys::{Document, Element, Event, HtmlInputElement};
use wee_alloc::WeeAlloc;

use roc::{
    discriminant_InternalAttr, discriminant_InternalHtml, InternalAttr, InternalEvent,
    InternalHtml, InternalHtmlElementFields, R3,
};

use crate::ws::ReconnectingWebSocket;

#[global_allocator]
static ALLOC: WeeAlloc<'_> = WeeAlloc::INIT;

static MODEL: LazyLock<Arc<Mutex<roc::Model>>> = LazyLock::new(|| {
    let frontend_model = roc::frontend_init_for_host(0);
    Arc::new(Mutex::new(unsafe { roc::Model::init(frontend_model) }))
});

thread_local! {
    // Thread-local storage for the DomBuilder to keep event handlers alive
    static BUILDER_STORAGE: std::cell::RefCell<Option<DomBuilder>> = std::cell::RefCell::new(None);
    static WS: std::cell::RefCell<ReconnectingWebSocket> = std::cell::RefCell::new( ReconnectingWebSocket::new(get_ws_url()) );
}

#[wasm_bindgen]
pub fn run() {
    console::debug_1(&"Initializing app...".into());

    let window = web_sys::window().unwrap();
    let document = window.document().expect("could not get document");
    let body = document.body().expect("Could not get document body");
    body.set_id("root");

    // Initial render
    render_app();

    // Setup WebSocket
    WS.with(|ws: &RefCell<ReconnectingWebSocket>| {
        let mut ws = ws.borrow_mut();
        ws.connect().expect("Failed to connect to websocket");
        ws.set_onmessage(|message_event| {
            if let Some(data) = message_event.data().as_string() {
                let msg = frontend_decode_to_frontend_msg(data.as_bytes().into());
                update_model_and_rerender(msg);
            }
        });
    });
}

fn get_ws_url() -> String {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let protocol = if location.protocol().unwrap() == "https:" {
        "wss:"
    } else {
        "ws:"
    };
    let host = location.host().unwrap();
    format!("{}//{}/ws", protocol, host)
}

fn render_app() {
    let model = MODEL.lock().unwrap();

    let app_html = roc::frontend_view_for_host(model.clone().inner);

    match render_to_dom(app_html, "root") {
        Ok(_) => {
            console::log(
                [JsValue::from("Render successful")]
                    .iter()
                    .collect::<Array>()
                    .as_ref(),
            );
        }
        Err(e) => {
            console::log(
                [JsValue::from("Render error:"), e]
                    .iter()
                    .collect::<Array>()
                    .as_ref(),
            );
        }
    }
}

fn update_model_and_rerender(message: RocBox<()>) {
    // TODO: Update the model with the message
    {
        let UpdateResult {
            to_backend,
            model: updated_model,
        } = {
            let model = MODEL.lock().expect("Unable to get model");
            roc::frontend_update_for_host(model.clone().inner, message)
        };
        let updated_model = unsafe { Model::init(updated_model) };
        let mut model = MODEL
            .lock()
            .expect("Could not acquire lock for model for update");
        *model = updated_model;
        drop(model);
        console::debug(
            [
                JsValue::from("Rerendering"),
                format!("To backend {:?}", &to_backend).into(),
            ]
            .iter()
            .collect::<Array>()
            .as_ref(),
        );

        if let Ok(to_backend_msg) = to_backend.into() {
            WS.with(|ws: &RefCell<ReconnectingWebSocket>| {
                let ws = ws.borrow();
                ws.send_message(to_backend_msg.as_str()).unwrap();
            });
        }
    }

    // Trigger re-render
    render_app();
}

pub struct DomBuilder {
    document: Document,
    event_handlers: Vec<Closure<dyn FnMut(Event)>>,
}

impl DomBuilder {
    pub fn new(document: Document) -> Self {
        Self {
            document,
            event_handlers: Vec::new(),
        }
    }

    pub fn build_dom(
        &mut self,
        html: &InternalHtml,
        parent_element: &Element,
    ) -> Result<(), JsValue> {
        // Clear existing content
        parent_element.set_inner_html("");

        match html.discriminant() {
            discriminant_InternalHtml::Text => {
                let text_content = html.get_Text_f0();
                let text_node = self.document.create_text_node(text_content.as_str());
                parent_element.append_child(&text_node)?;
            }
            discriminant_InternalHtml::Element => {
                let element_data = html.get_Element_fields();
                let element = self.build_element(element_data)?;
                parent_element.append_child(&element)?;
            }
        }

        Ok(())
    }

    fn build_element(
        &mut self,
        element_data: &InternalHtmlElementFields,
    ) -> Result<web_sys::Node, JsValue> {
        let tag_name = &element_data.tag;
        let attrs = &element_data.attrs;
        let children = &element_data.children;

        // Create the element
        let element = self.document.create_element(&tag_name.as_str())?;

        // Apply attributes and event listeners
        for i in 0..attrs.len() {
            unsafe {
                let elements_ptr = attrs.as_ptr() as *mut u8;
                let attr: *mut InternalAttr =
                    elements_ptr.add(InternalAttr::size() as usize * i).cast();

                self.apply_attribute(&element, &*attr)?;
            }
        }

        // Add children recursively
        for child in children.iter() {
            match child.discriminant() {
                discriminant_InternalHtml::Text => {
                    let text_content = child.get_Text_f0();
                    let text_node = self.document.create_text_node(text_content.as_str());
                    element.append_child(&text_node)?;
                }
                discriminant_InternalHtml::Element => {
                    let element_data = child.get_Element_fields();
                    let child_node = self.build_element(element_data)?;
                    element.append_child(&child_node)?;
                }
            }
        }

        Ok(element.into())
    }

    fn apply_attribute(&mut self, element: &Element, attr: &InternalAttr) -> Result<(), JsValue> {
        match attr.discriminant() {
            discriminant_InternalAttr::Id => {
                element.set_attribute("id", &attr.borrow_Id().as_str())?;
            }
            discriminant_InternalAttr::Class => {
                element.set_attribute("class", &attr.borrow_Class().as_str())?;
            }
            discriminant_InternalAttr::Value => {
                element.set_attribute("value", &attr.borrow_Value().as_str())?;
                // Also set the property for input elements
                if let Some(input) = element.dyn_ref::<HtmlInputElement>() {
                    input.set_value(&attr.borrow_Value().as_str());
                }
            }
            discriminant_InternalAttr::Placeholder => {
                element.set_attribute("placeholder", &attr.borrow_Placeholder().as_str())?;
            }
            discriminant_InternalAttr::Type => {
                element.set_attribute("type", &attr.borrow_Type().as_str())?;
            }
            discriminant_InternalAttr::Name => {
                element.set_attribute("name", &attr.borrow_Name().as_str())?;
            }
            discriminant_InternalAttr::Href => {
                element.set_attribute("href", &attr.borrow_Href().as_str())?;
            }
            discriminant_InternalAttr::Src => {
                element.set_attribute("src", &attr.borrow_Src().as_str())?;
            }
            discriminant_InternalAttr::Alt => {
                element.set_attribute("alt", &attr.borrow_Alt().as_str())?;
            }
            discriminant_InternalAttr::Title => {
                element.set_attribute("title", &attr.borrow_Title().as_str())?;
            }
            discriminant_InternalAttr::Style => {
                element.set_attribute("style", &attr.borrow_Style().as_str())?;
            }
            discriminant_InternalAttr::Autocomplete => {
                element.set_attribute("autocomplete", &attr.borrow_Autocomplete().as_str())?;
            }
            discriminant_InternalAttr::Tabindex => {
                element.set_attribute("tabindex", &attr.borrow_Tabindex().to_string())?;
            }

            // Boolean attributes
            discriminant_InternalAttr::Disabled => {
                if attr.borrow_Disabled() {
                    element.set_attribute("disabled", "")?;
                }
            }
            discriminant_InternalAttr::Checked => {
                if attr.borrow_Checked() {
                    element.set_attribute("checked", "")?;
                    if let Some(input) = element.dyn_ref::<HtmlInputElement>() {
                        input.set_checked(true);
                    }
                }
            }
            discriminant_InternalAttr::Selected => {
                if attr.borrow_Selected() {
                    element.set_attribute("selected", "")?;
                }
            }
            discriminant_InternalAttr::Hidden => {
                if attr.borrow_Hidden() {
                    element.set_attribute("hidden", "")?;
                }
            }
            discriminant_InternalAttr::Readonly => {
                if attr.borrow_Readonly() {
                    element.set_attribute("readonly", "")?;
                }
            }
            discriminant_InternalAttr::Required => {
                if attr.borrow_Required() {
                    element.set_attribute("required", "")?;
                }
            }
            discriminant_InternalAttr::Multiple => {
                if attr.borrow_Multiple() {
                    element.set_attribute("multiple", "")?;
                }
            }

            // Data attributes
            discriminant_InternalAttr::DataAttribute => {
                let data_attr = attr.borrow_DataAttribute();
                let key = format!("data-{}", data_attr.f0.as_str());
                element.set_attribute(&key, &data_attr.f1.as_str())?;
            }

            // Custom attributes
            discriminant_InternalAttr::Attribute => {
                let custom_attr = attr.borrow_Attribute();
                element.set_attribute(&custom_attr.f0.as_str(), &custom_attr.f1.as_str())?;
            }

            // Event handlers - This is where the magic happens for rerenders!
            discriminant_InternalAttr::OnEvent => {
                let event = attr.borrow_OnEvent();
                let event_type = event.event_type();
                let mut cb = event.event_callback();

                let closure = Closure::wrap(Box::new(move |e: Event| {
                    let internal_event = convert_web_event_to_internal(&e);
                    let message = cb.force_thunk(internal_event);

                    update_model_and_rerender(message);
                }) as Box<dyn FnMut(Event)>);

                element.add_event_listener_with_callback(
                    event_type.as_str(),
                    closure.as_ref().unchecked_ref(),
                )?;

                // Store the closure to keep it alive
                self.event_handlers.push(closure);
            }
        }
        Ok(())
    }
}

fn convert_web_event_to_internal(event: &Event) -> InternalEvent {
    // Get target information
    let target = get_target_info(event.target().as_ref());
    let current_target = get_target_info(event.current_target().as_ref());

    // Handle keyboard events
    let (key, code) = if let Some(keyboard_event) = event.dyn_ref::<web_sys::KeyboardEvent>() {
        (keyboard_event.key(), keyboard_event.code())
    } else {
        ("".to_string(), "".to_string())
    };

    // Handle mouse events
    let (button, client_x, client_y, alt_key, ctrl_key, shift_key, meta_key) =
        if let Some(mouse_event) = event.dyn_ref::<web_sys::MouseEvent>() {
            (
                mouse_event.button(),
                mouse_event.client_x(),
                mouse_event.client_y(),
                mouse_event.alt_key(),
                mouse_event.ctrl_key(),
                mouse_event.shift_key(),
                mouse_event.meta_key(),
            )
        } else if let Some(keyboard_event) = event.dyn_ref::<web_sys::KeyboardEvent>() {
            (
                0,
                0,
                0,
                keyboard_event.alt_key(),
                keyboard_event.ctrl_key(),
                keyboard_event.shift_key(),
                keyboard_event.meta_key(),
            )
        } else {
            (0, 0, 0, false, false, false, false)
        };

    InternalEvent {
        eventType: roc_std::RocStr::from(event.type_().as_str()),
        target,
        currentTarget: current_target,
        key: roc_std::RocStr::from(key.as_str()),
        code: roc_std::RocStr::from(code.as_str()),
        button: button.into(),
        clientX: client_x,
        clientY: client_y,
        altKey: alt_key,
        ctrlKey: ctrl_key,
        shiftKey: shift_key,
        metaKey: meta_key,
        preventDefault: false,
        stopPropagation: false,
    }
}

fn get_target_info(target: Option<&web_sys::EventTarget>) -> R3 {
    if let Some(target) = target {
        if let Some(element) = target.dyn_ref::<Element>() {
            let id = element.get_attribute("id").unwrap_or_default();
            let tag_name = element.tag_name().to_lowercase();

            let (value, checked) = if let Some(input) = element.dyn_ref::<HtmlInputElement>() {
                (input.value(), input.checked())
            } else {
                (String::new(), false)
            };

            R3 {
                id: roc_std::RocStr::from(id.as_str()),
                tagName: roc_std::RocStr::from(tag_name.as_str()),
                value: roc_std::RocStr::from(value.as_str()),
                checked,
            }
        } else {
            R3::default()
        }
    } else {
        R3::default()
    }
}

pub fn render_to_dom(html: InternalHtml, container_id: &str) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let container = document
        .get_element_by_id(container_id)
        .ok_or("Container element not found")?;

    let mut builder = DomBuilder::new(document);
    builder.build_dom(&html, &container)?;

    // Keep the builder alive to maintain event handlers
    // We need to store this somewhere so event handlers don't get dropped
    BUILDER_STORAGE.with(|storage| {
        *storage.borrow_mut() = Some(builder);
    });

    Ok(())
}
