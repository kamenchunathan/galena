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

FrontendModel : Dict Str I32
BackendModel : {}

ToFrontendMsg : {}
ToBackendMsg : {}

FrontendMsg : Str

frontendApp : Frontend FrontendModel FrontendMsg ToFrontendMsg ToBackendMsg
frontendApp = Frontend.frontend {
    init: Dict.empty {},
    update: |name, model|
        Dict.update
            model
            name
            (|entry|
                when entry is
                    Ok n -> Ok (n + 1)
                    Err Missing -> Ok 1
            ),

    view: |model|
        Dict.to_list model
        |> List.map
            (|(k, v)|
                s = if v == 1 then "" else "s"
                "You've greeted ${k}, ${Num.to_str v} time${s}")
        |> Str.join_with "\n"
        |> Str.concat "\n\n"
        |> View.text,

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

