module [View, Attr, text, div, input, id, class, value, placeholder]

import InternalView exposing [
    InternalAttr,
    InternalView,
    text_,
    div_,
    input_,
    id_,
    class_,
    value_,
    placeholder_,
]

Attr msg : InternalAttr msg

View msg : InternalView msg

text = text_

input = input_

div = div_

id = id_

class = class_

value = value_

placeholder = placeholder_
