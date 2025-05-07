module [
    InternalAttr,
    InternalView,
    repr_,
    text_,
    input_,
    div_,
    id_,
    class_,
    value_,
    placeholder_,
    internalViewToEncoder,
]

import json.Json as Json

InternalAttr msg := [
    OnInput (Str -> msg),
    OnEnter (Str -> msg),
    Id Str,
    Class Str,
    Value Str,
    Placeholder Str,
]
    implements [Encoding { to_encoder: internalAttrToEncoder }]

internalAttrToEncoder : InternalAttr msg -> Encoder fmt where fmt implements EncoderFormatting
internalAttrToEncoder = |attr|
    Encode.custom
        (|bytes, fmt|
            when attr is
                @InternalAttr (OnInput _) ->
                    Encode.append_with
                        bytes
                        (Encode.tuple [Encode.string "oninput", Encode.string "#unimplemented"])
                        fmt

                @InternalAttr (OnEnter _) ->
                    Encode.append_with
                        bytes
                        (Encode.tuple [Encode.string "oninput", Encode.string "#unimplemented"])
                        fmt

                @InternalAttr (Id id) ->
                    Encode.append_with bytes (Encode.tuple [Encode.string "id", Encode.string id]) fmt

                @InternalAttr (Class class) ->
                    Encode.append_with bytes (Encode.tuple [Encode.string "class", Encode.string class]) fmt

                @InternalAttr (Value val) ->
                    Encode.append_with bytes (Encode.tuple [Encode.string "val", Encode.string val]) fmt

                @InternalAttr (Placeholder placeholder) ->
                    Encode.append_with bytes (Encode.tuple [Encode.string "placeholder", Encode.string placeholder]) fmt
        )

InternalView msg := [
    Text Str,
    Div (List (InternalAttr msg)) (List (InternalView msg)),
    Input (List (InternalAttr msg)),
]
    implements [Encoding { to_encoder: internalViewToEncoder }]

internalViewToEncoder : InternalView msg -> Encoder fmt where fmt implements EncoderFormatting
internalViewToEncoder = |view|
    Encode.custom
        (|bytes, fmt|
            when view is
                @InternalView (Text t) ->
                    Encode.append_with
                        bytes
                        (Encode.record [{ key: "Text", value: Encode.string t }])
                        fmt

                @InternalView (Div attrs children) ->
                    Encode.append_with
                        bytes
                        (
                            Encode.record [
                                {
                                    key: "Div",
                                    value: Encode.record [
                                        {
                                            key: "attributes",
                                            value: Encode.list
                                                attrs
                                                (|internal_attr| internalAttrToEncoder internal_attr),
                                        },
                                        {
                                            key: "children",
                                            value: Encode.list
                                                children
                                                (|inner| internalViewToEncoder inner),
                                        },
                                    ],
                                },
                            ]
                        )
                        fmt

                @InternalView (Input attrs) ->
                    Encode.append_with
                        bytes
                        (
                            Encode.record [
                                {
                                    key: "Input",
                                    value: Encode.record [
                                        {
                                            key: "attributes",
                                            value: Encode.list
                                                attrs
                                                (|internal_attr| internalAttrToEncoder internal_attr),
                                        },
                                    ],
                                },
                            ]
                        )
                        fmt

        )

repr_ : InternalView msg -> List U8
repr_ = |view| Encode.to_bytes view Json.utf8

text_ : Str -> InternalView msg
text_ = |t| @InternalView (Text t)

div_ : List (InternalAttr msg), List (InternalView msg) -> InternalView msg
div_ = |attrs, children| @InternalView (Div attrs children)

input_ : List (InternalAttr msg) -> InternalView msg
input_ = |attrs| @InternalView (Input attrs)

id_ = |id| @InternalAttr (Id id)

placeholder_ = |placeholder| @InternalAttr (Placeholder placeholder)

value_ = |val| @InternalAttr (Value val)

class_ = |class| @InternalAttr (Class class)
