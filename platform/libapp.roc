app [
    FrontendModel,
    BackendModel,
    ToFrontendMsg,
    ToBackendMsg,
    frontendApp,
    backendApp,
] { galena: platform "../platform/main.roc" }

import galena.Backend as Backend exposing [Backend]
import galena.Frontend as Frontend exposing [Frontend]
import galena.Cmd as Cmd
import galena.View as View

FrontendModel : {}
BackendModel : {}

ToFrontendMsg : []
ToBackendMsg : []

frontendApp : Frontend FrontendModel {} ToFrontendMsg ToBackendMsg
frontendApp = Frontend.frontend {
    init: {},
    update: \_, model -> (model, Cmd.none),
    view: \_ -> View.text "wow",
    updateFromBackend: \_ -> Cmd.none,
}

backendApp : Backend BackendModel {} ToFrontendMsg ToBackendMsg
backendApp = Backend.backend {
    init: {},
    update: \_, model -> (model, Cmd.none),
    updateFromFrontend: \_ -> Cmd.none,
}

