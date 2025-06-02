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
        frontend_init_for_host!,
        frontend_update_for_host!,
        frontend_handle_ws_event_for_host!,
        frontend_view_for_host!,
        backend_init_for_host!,
        backend_update_for_host!,
    ]

import Backend exposing [Backend]
import Frontend exposing [Frontend]
import InternalBackend
import InternalFrontend
import InternalView

frontend_init_for_host! : I32 => Box FrontendModel
frontend_init_for_host! = |_|
    Box.box (InternalFrontend.inner frontendApp).init!

frontend_update_for_host! : Box FrontendModel, Box FrontendMsg => { model : Box FrontendModel, 
                                        to_backend : Result Str [NoOp] }
frontend_update_for_host! = |boxed_model, boxed_msg|
    model = Box.unbox boxed_model
    app = InternalFrontend.inner frontendApp
    msg = Box.unbox boxed_msg

    (updated_model, m_to_backend_msg) = app.update msg model
    {
        model: Box.box updated_model,
        to_backend: Result.map_ok
            m_to_backend_msg
            (|to_backend_msg|
                app.encode_to_backend_msg to_backend_msg
                |> Str.from_utf8_lossy
            ),
    }

frontend_handle_ws_event_for_host! : Box FrontendModel, List U8 => { model : Box FrontendModel, to_backend : Result Str [NoOp] }
frontend_handle_ws_event_for_host! = |boxed, msg_bytes|
    model = Box.unbox boxed
    app = InternalFrontend.inner frontendApp
    (updated_model, m_to_backend_msg) =
        app.decode_to_frontend_msg msg_bytes
        |> app.updateFromBackend
        |> app.update model
    {
        model: Box.box updated_model,
        to_backend: Result.map_ok
            m_to_backend_msg
            (|to_backend_msg|
                app.encode_to_backend_msg to_backend_msg
                |> Str.from_utf8_lossy
            ),
    }

frontend_view_for_host! : Box FrontendModel => { 
                model : Box FrontendModel, 
            view : List U8 ,  callback : U64 -> Box FrontendMsg }
frontend_view_for_host! = |boxed|
    model = Box.unbox boxed
    (encoded, _) =
        (InternalFrontend.inner frontendApp).view model
        |> InternalView.repr_

    {
        model: boxed,
        view:  encoded,
        callback: handle_dom_event model,
    }

handle_dom_event : FrontendModel -> (U64 -> Box FrontendMsg)
handle_dom_event = |model|
    |callback_id|
        app = InternalFrontend.inner frontendApp
        (_, callbacks) =
            app.view model
            |> InternalView.repr_
        when List.get callbacks callback_id is
            Ok cb ->
                # TODO: Replace this with  a proper event type
                Box.box (cb {})
            Err _ ->
                crash "Callback list is empty"

backend_init_for_host! : Box BackendModel
backend_init_for_host! =
    (InternalBackend.inner backendApp).init!
    |> Box.box

backend_update_for_host! : Box BackendModel, Str, Str, Str => { model : Box BackendModel, to_frontend : Result (Str, Str) [NoOp] }
backend_update_for_host! = |boxed_model, client_id, session_id, msg_bytes|
    model = Box.unbox boxed_model
    app = InternalBackend.inner backendApp

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
                (cid, msg)
            ),
    }

