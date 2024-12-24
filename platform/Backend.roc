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
        updateFromFrontend : toBackendMsg -> Cmd msg,
    }
    -> Backend model msg toFrontendMsg toBackendMsg
backend =
    @Backend

