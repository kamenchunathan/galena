module [InternalFrontend, frontend_, inner]

import View exposing [View]
import json.Json as Json

InternalFrontend model msg toFrontendMsg toBackendMsg := {
    init! : model,
    update! : msg, model => (model, Result toBackendMsg [NoOp]),
    view : model -> View msg,
    updateFromBackend : toFrontendMsg -> msg,
    encode_to_backend_msg : toBackendMsg -> List U8,
    decode_to_frontend_msg : List U8 -> toFrontendMsg,
}

FrontendAppSpec model msg toFrontendMsg toBackendMsg : {
    init! : model,
    update! : msg, model => (model, Result toBackendMsg [NoOp]),
    view : model -> View msg,
    updateFromBackend : toFrontendMsg -> msg,
}

frontend_ : FrontendAppSpec model msg toFrontendMsg toBackendMsg -> InternalFrontend model msg toFrontendMsg toBackendMsg where msg implements Decoding, toBackendMsg implements Encoding, toFrontendMsg implements Decoding
frontend_ = |orig|
    @InternalFrontend {
        init!: orig.init!,
        update!: orig.update!,
        view: orig.view,
        updateFromBackend: orig.updateFromBackend,
        encode_to_backend_msg: |to_backend_msg| Encode.to_bytes to_backend_msg Json.utf8,
        decode_to_frontend_msg: |msg_bytes|
            when Decode.from_bytes msg_bytes Json.utf8 is
                Ok msg ->
                    msg

                Err _ ->
                    crash "Unable to decode toFrontendMsg This is is a platform bug",
    }

inner = |@InternalFrontend i| i
