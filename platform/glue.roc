platform "galena_platform"
    requires {} {
    }
    exposes []
    packages {
    }
    imports []
    provides [
        frontend_view_for_host,
        frontend_init_for_host,
        frontend_update_for_host
    ]

import Internal.Html as Html

frontend_init_for_host : U32 -> U32
frontend_init_for_host = |model| model

frontend_view_for_host : U32 -> Html.InternalHtml U32
frontend_view_for_host = |_| Html.text_ ""

frontend_update_for_host : U32, U32 ->
    {
        model : U32,
        to_backend : Result Str [NoOp],
    }
frontend_update_for_host = |boxed_model, _|
    {
        model: boxed_model,
        to_backend: Ok ""
    }
