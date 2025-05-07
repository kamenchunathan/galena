module [
    View,
    Attr,
    text,
    div,
    button,
    id,
    class,
    value,
    placeholder,
    on_click,
]

import InternalView exposing [
    InternalAttr,
    InternalView,
    text_,
    div_,
    button_,
    id_,
    class_,
    value_,
    placeholder_,
    on_click_,
]

Attr msg : InternalAttr msg

View msg : InternalView msg

text = text_

button = button_

div = div_

id = id_

class = class_

value = value_

placeholder = placeholder_

on_click = on_click_

