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
import galena.Html as Html

FrontendModel : Str

BackendModel : {
    counter : I32,
}

ToFrontendMsg : I32

ToBackendMsg : I32

FrontendMsg : [Click, NoOp]

BackendendMsg : [UpdateCounter Str I32]

frontendApp : Frontend FrontendModel FrontendMsg ToFrontendMsg ToBackendMsg
frontendApp = Frontend.frontend {
    init!: "pong",

    update!: frontend_update!,

    view: view,

    updateFromBackend: |_| NoOp,
}

frontend_update! : FrontendMsg, FrontendModel => (FrontendModel, Result ToBackendMsg [NoOp])
frontend_update! = |msg, model|
    # print! (Inspect.to_str msg)
    when msg is
        Click ->
            ("Clicked", Err NoOp)

        NoOp -> (model, Err NoOp)

view : FrontendModel -> Html.Html FrontendMsg
view = |model|
    Html.div
        [Html.id "main", Html.class "bg-red-400 text-xl font-semibold"]
        [
            Html.div [] [
                Html.text model,
                Html.button
                    [
                        Html.id "incr",
                        Html.class "bg-slate-400 border-1 border-blue-400 outline-none",
                        Html.on_click (|_| Click),
                    ]
                    [Html.text "ping"],
            ],
        ]

backendApp : Backend BackendModel BackendendMsg ToFrontendMsg ToBackendMsg
backendApp = Backend.backend {
    init!: { counter: 0 },
    update!: |msg, model|
        when msg is
            UpdateCounter client_id client_counter ->
                (
                    { counter: model.counter + client_counter },
                    Ok (client_id, model.counter + client_counter),
                ),
    update_from_frontend: update_from_frontend,
}

update_from_frontend : Str, Str, ToBackendMsg -> BackendendMsg
update_from_frontend = |client_id, _, client_counter| UpdateCounter client_id client_counter

