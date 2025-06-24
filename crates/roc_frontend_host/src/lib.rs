mod roc;

use std::sync::{Arc, LazyLock, Mutex};

use wasm_bindgen::prelude::*;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::js_sys::Array;
use web_sys::{self, console};
use web_sys::{Document, Element, Event, HtmlElement, HtmlInputElement};
use wee_alloc::WeeAlloc;

use roc_app::{
    discriminant_InternalAttr, discriminant_InternalHtml, InternalAttr, InternalEvent,
    InternalHtml, R1, R2,
};

use roc::call_roc_view;
// use roc::call_roc_init;

#[global_allocator]
static ALLOC: WeeAlloc<'_> = WeeAlloc::INIT;

// static MODEL: OnceLock<roc::Model> = OnceLock::new();
// static CAPTURES: OnceLock<roc::Model> = OnceLock::new();
static MODEL: LazyLock<Arc<Mutex<roc::Model>>> = LazyLock::new(|| {
    let model = roc::call_roc_init();
    Arc::new(Mutex::new(model))
});

#[wasm_bindgen]
pub fn run() {
    // if let Ok(model) = MODEL.lock() {
    //     call_roc_view((*model).clone());
    // }

    let window = web_sys::window().unwrap();
    let document = window.document().expect("could not get document");
    let body = document.body().expect("Could not get document body");
    body.set_id("root");

    let model = roc_app::frontend_init_for_host(0);
    let app_html = roc_app::frontend_view_for_host(model);

    match render_to_dom(app_html, "root") {
        Ok(_) => {}
        Err(e) => {
            console::log([e].iter().collect::<Array>().as_ref());
        }
    }
    // if let Some(model) = MODEL.get().cloned() {
    //     let ViewResult {
    //         model,
    //         captures,
    //         view,
    //     } = call_roc_view(model);
    //
    //     MODEL.set(model);
    //     CAPTURES.set(captures);
    // }
}

pub struct DomBuilder {
    document: Document,
    event_handlers: Vec<Closure<dyn FnMut(Event)>>,
    current_element: Element,
}

impl DomBuilder {
    pub fn new(document: Document, root_element: Element) -> Self {
        Self {
            document,
            event_handlers: Vec::new(),
            current_element: root_element,
        }
    }

    pub fn build_dom(
        &mut self,
        html: &InternalHtml,
        parent_element: &Element,
    ) -> Result<(), JsValue> {
        match html.discriminant() {
            discriminant_InternalHtml::Text => {
                let text_content = html.get_Text_f0();
                parent_element.set_text_content(Some(text_content.as_str()));
            }
            discriminant_InternalHtml::Element => {
                let element_data = html.get_Element_f0();
                let tag_name = &element_data.tag;
                let attrs = &element_data.attrs;
                let children = &element_data.children;

                // Create the element
                let element = self.document.create_element(&tag_name.as_str())?;

                // Apply attributes and event listeners
                for attr in attrs.iter() {
                    self.apply_attribute(&element, attr)?;
                }

                parent_element.append_child(&element)?;

                // Add children recursively
                for child in children.iter() {
                    self.build_dom(child, &element)?;
                }
            }
        }

        Ok(())
    }

    fn build_element(&mut self, element_data: &R1) -> Result<web_sys::Node, JsValue> {
        let tag_name = &element_data.tag;
        let attrs = &element_data.attrs;
        let children = &element_data.children;

        // Create the element
        let element = self.document.create_element(&tag_name.as_str())?;

        // Apply attributes and event listeners
        for attr in attrs.iter() {
            self.apply_attribute(&element, attr)?;
        }

        // Add children recursively
        for child in children.iter() {
            match child.discriminant() {
                discriminant_InternalHtml::Text => {
                    let text_content = child.get_Text_f0();
                    self.current_element
                        .set_text_content(Some(text_content.as_str()));
                }
                discriminant_InternalHtml::Element => {
                    let element_data = child.get_Element_f0();
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

            // Event handlers
            discriminant_InternalAttr::OnEvent => {
                let event_data = attr.borrow_OnEvent().clone();
                let event_type = event_data.f0.as_str();

                let closure = Closure::wrap(Box::new(move |event: Event| {
                    let internal_event = convert_web_event_to_internal(&event);
                    let handler = event_data.f1.clone();
                    handler.force_thunk(internal_event);
                }) as Box<dyn FnMut(Event)>);

                element.add_event_listener_with_callback(
                    event_type,
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

fn get_target_info(target: Option<&web_sys::EventTarget>) -> R2 {
    if let Some(target) = target {
        if let Some(element) = target.dyn_ref::<Element>() {
            let id = element.get_attribute("id").unwrap_or_default();
            let tag_name = element.tag_name().to_lowercase();

            let (value, checked) = if let Some(input) = element.dyn_ref::<HtmlInputElement>() {
                (input.value(), input.checked())
            } else {
                (String::new(), false)
            };

            R2 {
                id: roc_std::RocStr::from(id.as_str()),
                tagName: roc_std::RocStr::from(tag_name.as_str()),
                value: roc_std::RocStr::from(value.as_str()),
                checked,
            }
        } else {
            R2::default()
        }
    } else {
        R2::default()
    }
}

pub fn render_to_dom(html: InternalHtml, container_id: &str) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let container = document
        .get_element_by_id(container_id)
        .ok_or("Container element not found")?;

    let mut builder = DomBuilder::new(document, container.clone());

    container.set_inner_html("");
    builder.build_dom(&html, &container)?;
    console::log([JsValue::from("Bingo")].iter().collect::<Array>().as_ref());

    std::mem::forget(builder);

    Ok(())
}
