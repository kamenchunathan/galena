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

FrontendModel : { counter : I32 }

BackendModel : {
    counter : I32,
}

ToFrontendMsg : I32

ToBackendMsg : I32

FrontendMsg : [Increment, NoOp]

BackendendMsg : [UpdateCounter I32]

frontendApp : Frontend FrontendModel FrontendMsg ToFrontendMsg ToBackendMsg
frontendApp = Frontend.frontend {
    init!: { counter: 0 },

    update: frontend_update,

    view: view,

    updateFromBackend: |_| NoOp,
}

frontend_update : FrontendMsg, FrontendModel -> (FrontendModel, Result ToBackendMsg [NoOp])
frontend_update = |msg, model|
    when msg is
        Increment ->
            incr = model.counter + 1
            ({ counter: incr }, Ok incr)

        NoOp -> (model, Err NoOp)

view : FrontendModel -> View.View FrontendMsg
view = |model|
    View.div
        [View.id "main", View.class "bg-red-400 text-xl font-semibold"]
        [
            View.div [] [
                View.text (Num.to_str model.counter),
                View.button
                    [
                        View.id "incr",
                        View.class "bg-slate-400 border-1 border-blue-400 outline-none",
                        View.on_click Increment,
                    ]
                    [View.text "+"],
            ],
        ]

backendApp : Backend BackendModel BackendendMsg ToFrontendMsg ToBackendMsg
backendApp = Backend.backend {
    init!: { counter: 0 },
    update!: |msg, model|
        when msg is
            UpdateCounter client_counter ->
                (
                    { counter: model.counter + client_counter },
                    Ok (model.counter + client_counter),
                ),
    update_from_frontend: update_from_frontend,
}

update_from_frontend : Str, Str, ToBackendMsg -> BackendendMsg
update_from_frontend = |_, _, client_counter| UpdateCounter client_counter

