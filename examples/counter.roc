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

FrontendModel : {
    local_clicks: U32,
    total_clicks: U32
}

BackendModel : {
    counter : I32,
}

ToFrontendMsg : I32

ToBackendMsg : I32

FrontendMsg : [Click, NoOp]

BackendendMsg : [UpdateCounter Str I32]

frontendApp : Frontend FrontendModel FrontendMsg ToFrontendMsg ToBackendMsg
frontendApp = Frontend.frontend {
    init!: { local_clicks: 0, total_clicks: 0 },

    update!: frontend_update!,

    view: view,

    updateFromBackend: |_| NoOp,
}

frontend_update! : FrontendMsg, FrontendModel => (FrontendModel, Result ToBackendMsg [NoOp])
frontend_update! = |msg, model|
    # print! (Inspect.to_str msg)
    when msg is
        Click ->
            (
                { 
                    local_clicks: model.local_clicks + 1,  
                    total_clicks: model.total_clicks 
                }, 
                Err NoOp
            )

        NoOp -> (model, Err NoOp)

view : FrontendModel -> Html.Html FrontendMsg
view = |{local_clicks, total_clicks}|
    Html.div
        [
            Html.style "display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 100vh; background-color: #f0f4f8; font-family: sans-serif; ",
        ]
        [
            Html.h1
                [ Html.style "color: #333; margin-bottom: 2rem;" ]
                [ Html.text "Counter Example" ],
            Html.div
                [ Html.style "background-color: #ffffff; padding: 2rem; border-radius: 8px; box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); text-align: center; " ]
                [
                    Html.div [ Html.style "margin-bottom: 1.5rem;" ] [
                        Html.span [ Html.style "font-weight: bold; color: #555;" ] [ Html.text "Your Clicks: " ],
                        Html.span [ Html.style "font-size: 1.2em; color: #007bff;" ] [ Html.text (Num.to_str local_clicks) ],
                    ],
                    Html.div [ Html.style "margin-bottom: 1.5rem;" ] [
                        Html.span [ Html.style "font-weight: bold; color: #555;" ] [ Html.text "Total Clicks: " ],
                        Html.span [ Html.style "font-size: 1.2em; color: #28a745;" ] [ Html.text (Num.to_str total_clicks) ],
                    ],
                    Html.button
                        [
                            Html.id "incr",
                            Html.style "background-color: #007bff; color: white; border: none; padding: 10px 20px; border-radius: 5px; font-size: 1em; cursor: pointer; outline: none; transition: background-color 0.3s;",
                            Html.on_click (|_| Click),
                        ]
                        [ Html.text "Increment" ],
                ],
            Html.footer
                [ Html.style "margin-top: 2rem; color: #888; font-size: 0.9em;" ]
                [ Html.text "A simple counter application built with Galena and Roc." ],
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

