platform "galena_platform"
    requires { FrontendModel, BackendModel, ToFrontendMsg, ToBackendMsg } {
        frontendApp : Frontend FrontendModel frontendMsg ToFrontendMsg ToBackendMsg,
        backendApp : Backend BackendModel backendMsg ToFrontendMsg ToBackendMsg,
    }
    exposes [Frontend, Backend, Cmd]
    packages {}
    imports []
    provides [
        host_init!,
        host_update!,
        host_view!,
        backend_init_for_host!,
        backend_update_for_host!,
    ]

import Frontend exposing [Frontend, impl_get_update_fn, impl_get_init_fn, impl_get_view_fn]
import Backend exposing [Backend]
import BackendInternal exposing [inner]
import View

host_init! : I32 => Box FrontendModel
host_init! = |_| Box.box (impl_get_init_fn frontendApp)

host_update! : Box FrontendModel, Str => Box FrontendModel
host_update! = |model, msg_bytes|
    ret_model = (impl_get_update_fn frontendApp) msg_bytes (Box.unbox model)
    Box.box ret_model

host_view! : Box FrontendModel => { model : Box FrontendModel, view : Str }
host_view! = |model| {
    model: model,
    view: (impl_get_view_fn frontendApp) (Box.unbox model) |> View.to_str,
}

backend_init_for_host! : Box BackendModel
backend_init_for_host! =
    (inner backendApp).init!
    |> Box.box

backend_update_for_host! : Box BackendModel, Str, Str, Str => Box BackendModel
backend_update_for_host! = |box_model, client_id, session_id, msg_bytes|
    model = Box.unbox box_model
    from_frontend = (inner backendApp).update_from_frontend client_id session_id (Str.to_utf8 msg_bytes)
    (inner backendApp).update! from_frontend model
    |> Box.box

