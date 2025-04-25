platform "galena_platform"
    requires { FrontendModel, BackendModel, ToFrontendMsg, ToBackendMsg } {
        frontendApp : Frontend FrontendModel frontendMsg ToFrontendMsg ToBackendMsg,
        backendApp : Backend BackendModel backendMsg ToFrontendMsg ToBackendMsg,
    }
    exposes [Frontend, Backend, Cmd]
    packages {}
    imports []
    provides [host_init!, host_update!, host_view!]

import Frontend exposing [Frontend, impl_get_update_fn, impl_get_init_fn]
import Backend exposing [Backend]

host_init! : I32 => Box FrontendModel
host_init! = |_| Box.box (impl_get_init_fn frontendApp)

host_update! : Box FrontendModel, Str => Box FrontendModel
host_update! = |model, msg_bytes|
    ret_model = (impl_get_update_fn frontendApp) msg_bytes (Box.unbox model)
    Box.box ret_model

host_view! : Box FrontendModel => { model : Box FrontendModel, view : Str }
host_view! = |model| {
    model: model,
    view: Inspect.to_str (Box.unbox model),
}
