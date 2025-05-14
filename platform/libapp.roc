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
import galena.View as View

FrontendModel : { counter : I32 }
BackendModel : {}

ToFrontendMsg : {}
ToBackendMsg : {}
FrontendMsg : {}

frontendApp : Frontend FrontendModel FrontendMsg ToFrontendMsg ToBackendMsg
frontendApp = Frontend.frontend {
    init!: { counter: 42069 },
    update: |_, model| ({ counter: model.counter + 1 }, Ok {}),
    view: |{ counter }| View.text "H ${Num.to_str counter}",
    updateFromBackend: |_| {},
}

backendApp : Backend BackendModel {} ToFrontendMsg ToBackendMsg
backendApp = Backend.backend {
    init!: {},
    update!: |_, model| (model, Err NoOp),
    update_from_frontend: update_from_frontend,
}

update_from_frontend : Str, Str, toBackendMsg -> {}
update_from_frontend = |_, _, _| {}

