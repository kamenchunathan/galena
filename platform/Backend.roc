module [Backend, backend]

import Cmd exposing [Cmd]

Backend model msg toFrontendMsg toBackendMsg := {
    init : model,
    update : msg, model -> (model, Cmd msg),
    updateFromFrontend : toBackendMsg -> Cmd msg,
}

backend :
    {
        init : model,
        update : msg, model -> (model, Cmd msg),
        # TODO: Add a preconnect that may upgrade the connection or respond with an error
        # beforeConnect : Req -> Response,
        updateFromFrontend : toBackendMsg -> Cmd msg,
    }
    -> Backend model msg toFrontendMsg toBackendMsg
backend =
    @Backend

