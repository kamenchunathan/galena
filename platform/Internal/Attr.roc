module [
    InternalAttr,
    InternalEvent,
    map,

    # Attribute constructors
    id_,
    class_,
    value_,
    placeholder_,
    type_,
    name_,
    href_,
    src_,
    alt_,
    title_,
    disabled_,
    checked_,
    selected_,
    hidden_,
    readonly_,
    required_,
    multiple_,
    autocomplete_,
    tabindex_,
    style_,
    data_,
    # Event handlers
    on_click_,
    on_input_,
    on_change_,
    on_submit_,
    on_focus_,
    on_blur_,
    on_keydown_,
    on_keyup_,
    on_keypress_,
    on_mousedown_,
    on_mouseup_,
    on_mouseover_,
    on_mouseout_,
    on_mouseenter_,
    on_mouseleave_,
    on_load_,
    on_scroll_,
]

InternalAttr msg := [
    # Standard attributes
    Id Str,
    Class Str,
    Value Str,
    Placeholder Str,
    Type Str,
    Name Str,
    Href Str,
    Src Str,
    Alt Str,
    Title Str,

    # Boolean attributes
    Disabled Bool,
    Checked Bool,
    Selected Bool,
    Hidden Bool,
    Readonly Bool,
    Required Bool,
    Multiple Bool,

    # Other attributes
    Autocomplete Str,
    Tabindex I32,
    Style Str,
    DataAttribute Str Str, # data-key value

    # Events
    OnEvent Str (InternalEvent -> msg),

    # Custom attribute fallback
    Attribute Str Str,
]

map : InternalAttr oldMsg, (oldMsg -> newMsg) -> InternalAttr newMsg
map = |attr, mapper|
    when attr is
        @InternalAttr (OnEvent eventType handler) ->
            # Transform the event handler
            newHandler = |event|
                originalResult = handler event
                mapper originalResult

            @InternalAttr (OnEvent eventType newHandler)

        # All other attributes don't contain messages, so they can be safely "cast"
        @InternalAttr (Id value) -> @InternalAttr (Id value)
        @InternalAttr (Class value) -> @InternalAttr (Class value)
        @InternalAttr (Value value) -> @InternalAttr (Value value)
        @InternalAttr (Placeholder value) -> @InternalAttr (Placeholder value)
        @InternalAttr (Type value) -> @InternalAttr (Type value)
        @InternalAttr (Name value) -> @InternalAttr (Name value)
        @InternalAttr (Href value) -> @InternalAttr (Href value)
        @InternalAttr (Src value) -> @InternalAttr (Src value)
        @InternalAttr (Alt value) -> @InternalAttr (Alt value)
        @InternalAttr (Title value) -> @InternalAttr (Title value)
        @InternalAttr (Style value) -> @InternalAttr (Style value)
        @InternalAttr (Autocomplete value) -> @InternalAttr (Autocomplete value)
        @InternalAttr (Tabindex value) -> @InternalAttr (Tabindex value)
        @InternalAttr (Disabled value) -> @InternalAttr (Disabled value)
        @InternalAttr (Checked value) -> @InternalAttr (Checked value)
        @InternalAttr (Selected value) -> @InternalAttr (Selected value)
        @InternalAttr (Hidden value) -> @InternalAttr (Hidden value)
        @InternalAttr (Readonly value) -> @InternalAttr (Readonly value)
        @InternalAttr (Required value) -> @InternalAttr (Required value)
        @InternalAttr (Multiple value) -> @InternalAttr (Multiple value)
        @InternalAttr (DataAttribute key value) -> @InternalAttr (DataAttribute key value)
        @InternalAttr (Attribute key value) -> @InternalAttr (Attribute key value)

InternalEvent : {
    eventType : Str,
    target : {
        value : Str,
        checked : Bool,
        id : Str,
        tagName : Str,
    },
    currentTarget : {
        value : Str,
        checked : Bool,
        id : Str,
        tagName : Str,
    },
    key : Str, # For keyboard events
    code : Str, # For keyboard events
    button : I32, # For mouse events
    clientX : I32, # Mouse position
    clientY : I32, # Mouse position
    ctrlKey : Bool, # Modifier keys
    shiftKey : Bool,
    altKey : Bool,
    metaKey : Bool,
    preventDefault : Bool,
    stopPropagation : Bool,
}

# Standard attribute constructors
id_ : Str -> InternalAttr msg
id_ = |value| @InternalAttr(Id value)

class_ : Str -> InternalAttr msg
class_ = |value| @InternalAttr(Class value)

value_ : Str -> InternalAttr msg
value_ = |value| @InternalAttr(Value value)

placeholder_ : Str -> InternalAttr msg
placeholder_ = |value| @InternalAttr(Placeholder value)

type_ : Str -> InternalAttr msg
type_ = |value| @InternalAttr(Type value)

name_ : Str -> InternalAttr msg
name_ = |value| @InternalAttr(Name value)

href_ : Str -> InternalAttr msg
href_ = |value| @InternalAttr(Href value)

src_ : Str -> InternalAttr msg
src_ = |value| @InternalAttr(Src value)

alt_ : Str -> InternalAttr msg
alt_ = |value| @InternalAttr(Alt value)

title_ : Str -> InternalAttr msg
title_ = |value| @InternalAttr(Title value)

# Boolean attribute constructors
disabled_ : Bool -> InternalAttr msg
disabled_ = |value| @InternalAttr(Disabled value)

checked_ : Bool -> InternalAttr msg
checked_ = |value| @InternalAttr(Checked value)

selected_ : Bool -> InternalAttr msg
selected_ = |value| @InternalAttr(Selected value)

hidden_ : Bool -> InternalAttr msg
hidden_ = |value| @InternalAttr(Hidden value)

readonly_ : Bool -> InternalAttr msg
readonly_ = |value| @InternalAttr(Readonly value)

required_ : Bool -> InternalAttr msg
required_ = |value| @InternalAttr(Required value)

multiple_ : Bool -> InternalAttr msg
multiple_ = |value| @InternalAttr(Multiple value)

# Other attribute constructors
autocomplete_ : Str -> InternalAttr msg
autocomplete_ = |value| @InternalAttr(Autocomplete value)

tabindex_ : I32 -> InternalAttr msg
tabindex_ = |value| @InternalAttr(Tabindex value)

style_ : Str -> InternalAttr msg
style_ = |value| @InternalAttr(Style value)

data_ : Str, Str -> InternalAttr msg
data_ = |key, value| @InternalAttr(DataAttribute key value)

# Event handler constructors
on_click_ : (InternalEvent -> msg) -> InternalAttr msg
on_click_ = |handler| @InternalAttr(OnEvent "click" handler)

on_input_ : (InternalEvent -> msg) -> InternalAttr msg
on_input_ = |handler| @InternalAttr(OnEvent "input" handler)

on_change_ : (InternalEvent -> msg) -> InternalAttr msg
on_change_ = |handler| @InternalAttr(OnEvent "change" handler)

on_submit_ : (InternalEvent -> msg) -> InternalAttr msg
on_submit_ = |handler| @InternalAttr(OnEvent "submit" handler)

on_focus_ : (InternalEvent -> msg) -> InternalAttr msg
on_focus_ = |handler| @InternalAttr(OnEvent "focus" handler)

on_blur_ : (InternalEvent -> msg) -> InternalAttr msg
on_blur_ = |handler| @InternalAttr(OnEvent "blur" handler)

# Keyboard events
on_keydown_ : (InternalEvent -> msg) -> InternalAttr msg
on_keydown_ = |handler| @InternalAttr(OnEvent "keydown" handler)

on_keyup_ : (InternalEvent -> msg) -> InternalAttr msg
on_keyup_ = |handler| @InternalAttr(OnEvent "keyup" handler)

on_keypress_ : (InternalEvent -> msg) -> InternalAttr msg
on_keypress_ = |handler| @InternalAttr(OnEvent "keypress" handler)

# Mouse events
on_mousedown_ : (InternalEvent -> msg) -> InternalAttr msg
on_mousedown_ = |handler| @InternalAttr(OnEvent "mousedown" handler)

on_mouseup_ : (InternalEvent -> msg) -> InternalAttr msg
on_mouseup_ = |handler| @InternalAttr(OnEvent "mouseup" handler)

on_mouseover_ : (InternalEvent -> msg) -> InternalAttr msg
on_mouseover_ = |handler| @InternalAttr(OnEvent "mouseover" handler)

on_mouseout_ : (InternalEvent -> msg) -> InternalAttr msg
on_mouseout_ = |handler| @InternalAttr(OnEvent "mouseout" handler)

on_mouseenter_ : (InternalEvent -> msg) -> InternalAttr msg
on_mouseenter_ = |handler| @InternalAttr(OnEvent "mouseenter" handler)

on_mouseleave_ : (InternalEvent -> msg) -> InternalAttr msg
on_mouseleave_ = |handler| @InternalAttr(OnEvent "mouseleave" handler)

# Other events
on_load_ : (InternalEvent -> msg) -> InternalAttr msg
on_load_ = |handler| @InternalAttr(OnEvent "load" handler)

on_scroll_ : (InternalEvent -> msg) -> InternalAttr msg
on_scroll_ = |handler| @InternalAttr(OnEvent "scroll" handler)
