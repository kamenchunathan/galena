module [BackendInternal, backend_, inner]

import json.Json

BackendInternal model msg toFrontendMsg toBackendMsg := {
    init! : model,
    update! : msg, model => (model, toFrontendMsg),
    update_from_frontend : Str, Str, List U8 -> msg,
}

backend_ : { init! : model, update! : msg, model => (model, toFrontendMsg), update_from_frontend : Str, Str, toBackendMsg -> msg } -> BackendInternal model msg toFrontendMsg toBackendMsg where toBackendMsg implements Decoding
backend_ = |backend_config|
    @BackendInternal {
        init!: backend_config.init!,
        update!: backend_config.update!,
        update_from_frontend: |client_id, session_id, msg_bytes|
            to_backend_msg : toBackendMsg
            to_backend_msg =
                # TODO: This is okay for now since the types required by the platform are
                # constrained to be the same for both the frontendApp and backendApp but
                # find an alternative to crashing
                when Decode.from_bytes msg_bytes Json.utf8 is
                    Ok msg ->
                        msg

                    Err _ ->
                        crash "Unable to decode toBackendMsg"
            backend_config.update_from_frontend client_id session_id to_backend_msg,
    }

inner = |@BackendInternal(i)| i

