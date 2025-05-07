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
import galena.View as View

FrontendModel : { counter : Dict Str U32 }

BackendModel : {}

ToFrontendMsg : {}

ToBackendMsg : {}

FrontendMsg : {}

frontendApp : Frontend FrontendModel FrontendMsg ToFrontendMsg ToBackendMsg
frontendApp = Frontend.frontend {
    init!: { counter: Dict.empty {} },

    update: |_, model|
        (model, {}),

    view: |_|
        View.div
            [View.id "main", View.class "bg-red-400 text-xl font-semibold"]
            [View.div [] [View.text "This is a form"
            , View.input [View.id "name-input", View.class "bg-slate-400 border-1 border-blue-400 outlie-none", View.value "wow", View.placeholder "Name please"]]],
    updateFromBackend: |_| {},
}

backendApp : Backend BackendModel {} ToFrontendMsg ToBackendMsg
backendApp = Backend.backend {
    init!: {},
    update!: |_, model| (model, {}),
    update_from_frontend: update_from_frontend,
}

update_from_frontend : Str, Str, toBackendMsg -> {}
update_from_frontend = |_, _, _| {}

