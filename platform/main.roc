platform "galena_platform"
    requires { FrontendModel, BackendModel, ToFrontendMsg, ToBackendMsg, FrontendMsg } {
        frontendApp : Frontend FrontendModel FrontendMsg ToFrontendMsg ToBackendMsg,
        backendApp : Backend BackendModel backendMsg ToFrontendMsg ToBackendMsg,
    }
    exposes [Frontend, Backend]
    packages {
        json: "https://github.com/lukewilliamboswell/roc-json/releases/download/0.13.0/RqendgZw5e1RsQa3kFhgtnMP8efWoqGRsAvubx4-zus.tar.br",
    }
    imports []
    provides [
        frontend_decode_to_frontend_msg,
        frontend_init_for_host!,
        frontend_update_for_host!,
        frontend_view_for_host!,
        backend_init_for_host!,
        backend_update_for_host!,
    ]

import Backend exposing [Backend]
import Frontend exposing [Frontend]
import Internal.Backend
import Internal.Frontend
import Html

frontend_init_for_host! : I32 => Box FrontendModel
frontend_init_for_host! = |_|
    Box.box (Internal.Frontend.inner frontendApp).init!

frontend_update_for_host! :
    Box FrontendModel,
    Box FrontendMsg
    =>
    {
        model : Box FrontendModel,
        to_backend : Result Str [NoOp],
    }
frontend_update_for_host! = |boxed_model, boxed_msg|
    app = Internal.Frontend.inner frontendApp
    model = Box.unbox boxed_model
    msg = Box.unbox boxed_msg
    
    (updated_model, m_to_backend_msg) = app.update! msg model
    {
        model: Box.box updated_model,
        to_backend: Result.map_ok
            m_to_backend_msg
            (|to_backend_msg|
                app.encode_to_backend_msg to_backend_msg
                |> Str.from_utf8_lossy
            ),
    }

frontend_decode_to_frontend_msg : List U8 -> Box ToFrontendMsg
frontend_decode_to_frontend_msg = | msg_bytes|
    app = Internal.Frontend.inner frontendApp
    app.decode_to_frontend_msg msg_bytes
        |> Box.box
            
frontend_view_for_host! : Box FrontendModel => Html.Html (Result (Box FrontendMsg) {})
frontend_view_for_host! = |boxed|
    model = Box.unbox boxed
    app = Internal.Frontend.inner frontendApp
        
    # NOTE: This forces the alignment of the captured type to 8 so that calculations
    # on the size of the attribute which are dependent on size and alignment of captures
    # in the function returned by the onevent are easier as we dont have to account for
    # multiple alignment values. The possible values of this alignment are either 4 or 8
    # due to restrictions by other types, capturing a U64 ensures the strictest alignment
    # and hence the overall alignment is 8 bytes. with a downside of space efficiency 
    a : U64
    a = 0
    
    Html.div [] [ 
        app.view model |> Html.map (|msg| Ok (Box.box msg)),
        Html.div [ Html.on_click (|_| Err (drop a)) , Html.attribute "hidden" "" ] []
    ]


backend_init_for_host! : Box BackendModel
backend_init_for_host! =
    (Internal.Backend.inner backendApp).init!
    |> Box.box

#  NOTE: Currently only called when we receive a message
# TODO: Expand the circumstances in which this would be called e.g. with subscriptions
#  or rename this to be more descriptive of this specific scenario
backend_update_for_host! : 
    Box BackendModel, Str, Str, Str => 
    { 
        model: Box BackendModel, 
        to_frontend: Result { client_id: Str,  message: Str} [NoOp]
    }
backend_update_for_host! = |boxed_model, client_id, session_id, msg_bytes|
    model = Box.unbox boxed_model
    app = Internal.Backend.inner backendApp

    (updated_model, m_to_frontend_msg) =
        app.update_from_frontend
            client_id
            session_id
            (app.decode_to_backend_msg (Str.to_utf8 msg_bytes))
        |> app.update! model

    {
        model: Box.box updated_model,
        to_frontend: Result.map_ok
            m_to_frontend_msg
            (|(cid, to_frontend_msg)|
                msg = Str.from_utf8_lossy (app.encode_to_frontend_msg to_frontend_msg)
                { client_id: cid, message: msg }
            ),
    }

drop = |_| {}
