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
    ]

import Internal.Html as Html

frontend_init_for_host : U32 -> U32
frontend_init_for_host = |model| model

frontend_view_for_host : U32 -> Html.InternalHtml U32
frontend_view_for_host = |_| Html.text_ ""
