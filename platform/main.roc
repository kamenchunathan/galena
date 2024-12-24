platform "galena_platform"
    requires { FrontendModel, BackendModel, ToFrontendMsg, ToBackendMsg } {
        frontendApp : Frontend FrontendModel frontendMsg ToFrontendMsg ToBackendMsg,
        backendApp : Backend BackendModel backendMsg ToFrontendMsg ToBackendMsg,
    }
    exposes [Frontend, Backend, Cmd]
    packages {}
    imports []
    provides [mainForHost]

import Frontend exposing [Frontend]
import Backend exposing [Backend]

mainForHost : Str
mainForHost = const (frontendApp, backendApp) "Hello world"

const = \_, a -> a

