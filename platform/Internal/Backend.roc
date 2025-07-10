module [BackendInternal, backend_, inner]

import json.Json

BackendInternal model msg to_frontend_msg to_backend_msg := {
    init! : model,
    update! : msg, model => (model, Result (Str, to_frontend_msg) [NoOp]),
    update_from_frontend : Str, Str, to_backend_msg -> msg,
    encode_to_frontend_msg : to_frontend_msg -> List U8,
    decode_to_backend_msg : List U8 -> to_backend_msg,
}

InternalBackendAppSpec model msg to_frontend_msg to_backend_msg : {
    init! : model,
    update! : msg, model => (model, Result (Str, to_frontend_msg) [NoOp]),
    update_from_frontend : Str, Str, to_backend_msg -> msg,
}

backend_ : InternalBackendAppSpec model msg to_frontend_msg to_backend_msg -> BackendInternal model msg to_frontend_msg to_backend_msg where to_backend_msg implements Decoding, to_frontend_msg implements Encoding
backend_ = |backend_config|
    @BackendInternal {
        init!: backend_config.init!,
        update!: backend_config.update!,
        update_from_frontend: |client_id, session_id, to_backend_msg|
            backend_config.update_from_frontend client_id session_id to_backend_msg,
        encode_to_frontend_msg: |to_frontend_msg| Encode.to_bytes to_frontend_msg Json.utf8,
        decode_to_backend_msg: |msg_bytes|
            when Decode.from_bytes msg_bytes Json.utf8 is
                Ok msg ->
                    msg

                Err _ ->
                    crash "Unable to decode toBackendMsg this is a platform bug",
    }

inner = |@BackendInternal(i)| i

