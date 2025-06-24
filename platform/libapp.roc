app [
    FrontendModel,
    BackendModel,
    ToFrontendMsg,
    FrontendMsg,
    ToBackendMsg,
    frontendApp,
    backendApp,
] { galena: platform "./main.roc" }

import galena.Backend as Backend exposing [Backend]
import galena.Frontend as Frontend exposing [Frontend]
import galena.Html as Html

FrontendModel : {}
BackendModel : {}

ToFrontendMsg : {}
ToBackendMsg : {}
FrontendMsg : {}

frontendApp : Frontend FrontendModel FrontendMsg ToFrontendMsg ToBackendMsg
frontendApp = Frontend.frontend {
    init!: {},
    update!: frontend_update!,
    view: |_| Html.text "Hello World",
    updateFromBackend: |_| {},
}

frontend_update! = |_, _| ({}, Err NoOp)

backendApp : Backend BackendModel {} ToFrontendMsg ToBackendMsg
backendApp = Backend.backend {
    init!: {},
    update!: |_, model| (model, Err NoOp),
    update_from_frontend: update_from_frontend,
}

update_from_frontend : Str, Str, toBackendMsg -> {}
update_from_frontend = |_, _, _| {}

