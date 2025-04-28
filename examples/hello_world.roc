app [
    FrontendModel,
    BackendModel,
    ToFrontendMsg,
    FrontendMsg,
    ToBackendMsg,
    frontendApp,
    backendApp,
] { galena: platform "../platform/main.roc" }

import galena.Backend as Backend exposing [Backend]
import galena.Frontend as Frontend exposing [Frontend]
import galena.Cmd as Cmd
import galena.View as View

FrontendModel : { counter : I32 }
BackendModel : {}

ToFrontendMsg : []
ToBackendMsg : {}
FrontendMsg : Str

frontendApp : Frontend FrontendModel FrontendMsg ToFrontendMsg ToBackendMsg
frontendApp = Frontend.frontend {
    init: { counter: 42069 },
    update: |_, model| { counter: model.counter + 1 },
    view: |{ counter }| View.text "H ${Num.to_str counter}",
    updateFromBackend: |_| Cmd.none,
}

backendApp : Backend BackendModel {} ToFrontendMsg ToBackendMsg
backendApp = Backend.backend {
    init!: {},
    update!: |_, model| model,
    update_from_frontend: update_from_frontend,
}

update_from_frontend : Str, Str, toBackendMsg -> {}
update_from_frontend = |_, _, _| {}

