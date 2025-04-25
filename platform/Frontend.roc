module [Frontend, frontend, impl_get_init_fn, impl_get_update_fn, impl_get_view_fn]

import View exposing [View]
import Cmd exposing [Cmd]

Frontend model msg toFrontendMsg toBackendMsg := {
    init : model,
    update : msg, model -> model,
    view : model -> View msg,
    updateFromBackend : toFrontendMsg -> Cmd msg,
}

frontend :
    {
        init : model,
        update : msg, model -> model,
        view : model -> View msg,
        updateFromBackend : toFrontendMsg -> Cmd msg,
    }
    -> Frontend model msg toFrontendMsg toBackendMsg
frontend =
    @Frontend

# TODO: Maybe move these to an internal module

impl_get_init_fn : Frontend model msg toFrontendMsg toBackendMsg -> model
impl_get_init_fn = |@Frontend({ init })| init

impl_get_update_fn :
    Frontend model msg toFrontendMsg toBackendMsg
    -> (msg, model -> model)
impl_get_update_fn = |@Frontend({ update })| update

impl_get_view_fn :
    Frontend model msg toFrontendMsg toBackendMsg
    -> (model -> View msg)
impl_get_view_fn = |@Frontend({ view })| view
