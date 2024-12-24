module [Frontend, frontend]

import View exposing [View]
import Cmd exposing [Cmd]

Frontend model msg toFrontendMsg toBackendMsg := {
    init : model,
    update : msg, model -> (model, Cmd msg),
    view : model -> View msg,
    updateFromBackend : toFrontendMsg -> Cmd msg,
}

frontend :
    {
        init : model,
        update : msg, model -> (model, Cmd msg),
        view : model -> View msg,
        updateFromBackend : toFrontendMsg -> Cmd msg,
    }
    -> Frontend model msg toFrontendMsg toBackendMsg
frontend =
    @Frontend
