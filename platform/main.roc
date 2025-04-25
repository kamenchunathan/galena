platform "galena_platform"
    requires { FrontendModel, BackendModel, ToFrontendMsg, ToBackendMsg } {
        frontendApp : Frontend FrontendModel frontendMsg ToFrontendMsg ToBackendMsg,
        backendApp : Backend BackendModel backendMsg ToFrontendMsg ToBackendMsg,
    }
    exposes [Frontend, Backend, Cmd]
    packages {}
    imports []
    provides [host_init!, host_update!]

import Frontend exposing [Frontend, impl_get_update_fn, impl_get_init_fn]
import Backend exposing [Backend]

host_init! : I32 => Box FrontendModel
host_init! = |_| Box.box (impl_get_init_fn frontendApp)

host_update! : Box FrontendModel, Str => Str
host_update! = |model, msg_bytes|
    (ret_model, _) = (impl_get_update_fn frontendApp) msg_bytes (Box.unbox model)
    # "Hello world you intelligent person "
    "wow"
# Inspect.to_str ret_model

