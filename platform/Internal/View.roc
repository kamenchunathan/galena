module [
    InternalAttr,
    InternalView,
    repr_,
    text_,
    button_,
    div_,
    id_,
    class_,
    value_,
    placeholder_,
    on_click_,
]

import json.Json as Json

InternalAttr msg := [
    OnClick (Event -> msg),
    Id Str,
    Class Str,
    Value Str,
    Placeholder Str,
]

Event : {}

# NOTE: Some of the weird things I'm doing in the append_attr and append_view functions
# are due to problems encountered with Encoders being passed as is.
# They are numerous and not exhaustively recorded or the specific causes well understood
# but here's some of those encountered to keep in mind while refactoring
# 1. encoders that take lists of encoders result in borrow signature lambda errors. Hence
#   the redirection using custom and append_with
# 2. A concrete formatter type is required during recursion
append_attr : InternalAttr msg, List (Event -> msg) -> (Encoder Json.Json, List (Event -> msg)) where fmt implements EncoderFormatting
append_attr = |attr, callbacks|
    when attr is
        @InternalAttr (OnClick cb) ->
            (
                Encode.custom
                    (|bytes, fmt|
                        Encode.append_with
                            bytes
                            (
                                Encode.tuple
                                    [Encode.string "onclick", Encode.u64 (List.len callbacks)]
                            )
                            fmt
                    ),
                List.append callbacks cb,
            )

        @InternalAttr (Id id) ->
            (
                Encode.custom
                    (|bytes, fmt|
                        Encode.append_with bytes (Encode.tuple [Encode.string "id", Encode.string id]) fmt),
                callbacks,
            )

        @InternalAttr (Class class) ->
            (
                Encode.custom
                    (|bytes, fmt|
                        Encode.append_with bytes (Encode.tuple [Encode.string "class", Encode.string class]) fmt),
                callbacks,
            )

        @InternalAttr (Value val) ->
            (
                Encode.custom
                    (|bytes, fmt|
                        Encode.append_with bytes (Encode.tuple [Encode.string "val", Encode.string val]) fmt),
                callbacks,
            )

        @InternalAttr (Placeholder placeholder) ->
            (
                Encode.custom
                    (|bytes, fmt|
                        Encode.append_with bytes (Encode.tuple [Encode.string "placeholder", Encode.string placeholder]) fmt),
                callbacks,
            )

InternalView msg := [
    Text Str,
    Div (List (InternalAttr msg)) (List (InternalView msg)),
    Button (List (InternalAttr msg)) (List (InternalView msg)),
]

append_view :
    InternalView msg,
    List (Event -> msg)
    -> (Encoder Json.Json, List (Event -> msg))
append_view = |view, callbacks|
    when view is
        @InternalView (Text t) ->
            (
                Encode.custom
                    (|bytes, fmt|
                        Encode.append_with
                            bytes
                            (Encode.record [{ key: "Text", value: Encode.string t }])
                            fmt
                    ),
                callbacks,
            )

        @InternalView (Button attrs children) ->
            (attr_encoders, attr_cbs) = List.walk
                attrs
                ([], callbacks)
                (|(encs, cbs), attr|
                    (enc, updated_cbs) = append_attr attr cbs
                    (
                        List.append encs enc,
                        List.concat cbs updated_cbs,
                    )
                )
            attr_bytes = Encode.append_with
                []
                (Encode.tuple attr_encoders)
                Json.utf8

            (child_encoders, child_cbs) = List.walk
                children
                ([], attr_cbs)
                (|(encs, cbs), child_view|
                    (enc, updated_cbs) = append_view child_view cbs
                    (
                        List.append encs enc,
                        List.concat cbs updated_cbs,
                    )
                )
            child_bytes = Encode.append_with
                []
                (Encode.tuple child_encoders)
                Json.utf8

            encoded =
                Encode.record [
                    {
                        key: "Button",
                        value: Encode.record [
                            {
                                key: "attributes",
                                value: Encode.custom (|bytes, _| List.concat bytes attr_bytes),
                            },
                            {
                                key: "children",
                                value: Encode.custom (|bytes, _| List.concat bytes child_bytes),
                            },
                        ],
                    },
                ]

            (encoded, List.concat callbacks child_cbs)

        @InternalView (Div attrs children) ->
            (attr_encoders, attr_cbs) = List.walk
                attrs
                ([], callbacks)
                (|(encs, cbs), attr|
                    (enc, updated_cbs) = append_attr attr cbs
                    (
                        List.append encs enc,
                        List.concat cbs updated_cbs,
                    )
                )
            attr_bytes = Encode.append_with
                []
                (Encode.tuple attr_encoders)
                Json.utf8

            (child_encoders, child_cbs) = List.walk
                children
                ([], attr_cbs)
                (|(encs, cbs), child_view|
                    (enc, updated_cbs) = append_view child_view cbs
                    (
                        List.append encs enc,
                        List.concat cbs updated_cbs,
                    )
                )
            child_bytes = Encode.append_with
                []
                (Encode.tuple child_encoders)
                Json.utf8

            encoded =
                Encode.record [
                    {
                        key: "Div",
                        value: Encode.record [
                            {
                                key: "attributes",
                                value: Encode.custom (|bytes, _| List.concat bytes attr_bytes),
                            },
                            {
                                key: "children",
                                value: Encode.custom (|bytes, _| List.concat bytes child_bytes),
                            },
                        ],
                    },
                ]

            (encoded, List.concat callbacks child_cbs)

#        @InternalView (Input attrs) ->
#            (attr_encoders , attr_cbs) = List.walk
#                attrs
#                ([], [])
#                (|(encs, cbs), attr|
#                    (enc, updated_cbs) = append_attr attr cbs
#                    (List.append encs enc
#                    ,List.concat cbs updated_cbs
#                    )
#                )
#            attr_bytes = Encode.append_with
#                []
#                (Encode.tuple attr_encoders)
#                Json.utf8
#
#            (
#                Encode.record [
#                    {
#                        key: "Input",
#                        value: Encode.record [
#                            {
#                                key: "attributes",
#                                value: Encode.custom (|bytes, _| List.concat bytes attr_bytes)
#                            },
#                        ],
#                    },
#                ]
#                ,
#                List.concat callbacks attr_cbs
#
#            )

repr_ : InternalView msg -> (List U8, List (Event -> msg))
repr_ = |view|
    (view_encoder, callbacks) = append_view view []
    (
        Encode.append_with [] view_encoder Json.utf8,
        callbacks,
    )

text_ : Str -> InternalView msg
text_ = |t| @InternalView (Text t)

div_ : List (InternalAttr msg), List (InternalView msg) -> InternalView msg
div_ = |attrs, children| @InternalView (Div attrs children)

button_ : List (InternalAttr msg), List (InternalView msg) -> InternalView msg
button_ = |attrs, children| @InternalView (Button attrs children)

id_ = |id| @InternalAttr (Id id)

placeholder_ = |placeholder| @InternalAttr (Placeholder placeholder)

value_ = |val| @InternalAttr (Value val)

class_ = |class| @InternalAttr (Class class)

on_click_ = |ev| @InternalAttr (OnClick ev)
