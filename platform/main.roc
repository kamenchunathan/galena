platform "galena_platform"
    requires { FrontendModel, BackendModel, ToFrontendMsg, ToBackendMsg } {
        frontendApp : Frontend FrontendModel frontendMsg ToFrontendMsg ToBackendMsg,
        backendApp : Backend BackendModel backendMsg ToFrontendMsg ToBackendMsg,
    }
    exposes [Frontend, Backend]
    packages {
        json: "https://github.com/lukewilliamboswell/roc-json/releases/download/0.13.0/RqendgZw5e1RsQa3kFhgtnMP8efWoqGRsAvubx4-zus.tar.br",
        # msg_pack: "/home/nathankamenchu/dev/read_the_code/roc/roc-msgpack/msg_pack.tar.br",
    }
    imports []
    provides [
        frontend_host_init!,
        frontend_host_update!,
        frontend_host_view!,
        backend_init_for_host!,
        backend_update_for_host!,
        frontend_receive_ws_message_for_host!,
    ]

import Backend exposing [Backend]
import Frontend exposing [Frontend]
import Host exposing [send_to_backend_impl!]
import InternalBackend
import InternalFrontend
import InternalView

frontend_host_init! : I32 => Box FrontendModel
frontend_host_init! = |_| Box.box (InternalFrontend.inner frontendApp).init!

frontend_host_update! : Box FrontendModel, Str => Box FrontendModel
frontend_host_update! = |model, _msg_bytes|
    # TODO: We're actually not going to call the update function now because that requires
    # having set up a UI framework

    model

frontend_receive_ws_message_for_host! : Box FrontendModel, List U8 => Box FrontendModel
frontend_receive_ws_message_for_host! = |model, msg_bytes|
    app = InternalFrontend.inner frontendApp
    (updated_model, toBackendMsg) =
        app.decode_to_frontend_msg msg_bytes
        |> app.update (Box.unbox model)
    app.encode_to_backend_msg toBackendMsg
    |> Str.from_utf8_lossy
    |> send_to_backend_impl!
    Box.box updated_model

frontend_host_view! : Box FrontendModel => { model : Box FrontendModel, view : List U8 }
frontend_host_view! = |model| {
    model: model,
    view: (InternalFrontend.inner frontendApp).view (Box.unbox model) |> InternalView.repr_,
}

backend_init_for_host! : Box BackendModel
backend_init_for_host! =
    (InternalBackend.inner backendApp).init!
    |> Box.box

backend_update_for_host! : Box BackendModel, Str, Str, Str => Box BackendModel
backend_update_for_host! = |model, client_id, session_id, msg_bytes|
    from_frontend =
        (InternalBackend.inner backendApp).update_from_frontend
            client_id
            session_id
            (Str.to_utf8 msg_bytes)
    (updated_model, _) =
        (InternalBackend.inner backendApp).update!
            from_frontend
            (Box.unbox model)
    Box.box updated_model

