platform "galena_platform"
    requires { FrontendModel, BackendModel, ToFrontendMsg, ToBackendMsg, FrontendMsg } {
        frontendApp : Frontend FrontendModel FrontendMsg ToFrontendMsg ToBackendMsg,
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


HostModel :{ frontend_model : FrontendModel,
    callbacks: List FrontendMsg  } 


frontend_host_init! : I32 => Box  HostModel  
frontend_host_init! = |_| 
    Box.box {
        frontend_model: (InternalFrontend.inner frontendApp).init!, 
        callbacks: []
    }

frontend_host_update! : Box HostModel, U32, Str => Box HostModel
frontend_host_update! = |boxed, callback_id, _msg_bytes|
    model = Box.unbox boxed
    app = InternalFrontend.inner frontendApp
    
    when List.get model.callbacks (Num.to_u64 callback_id) is
        Ok cb ->  
            (updated_model, m_to_backend_msg) = app.update cb model.frontend_model
            _ = when m_to_backend_msg is 
                Ok to_backend_msg ->
                    app.encode_to_backend_msg to_backend_msg
                        |> Str.from_utf8_lossy
                        |> send_to_backend_impl!
                    0
                Err _ -> 0
                
            Box.box {
                frontend_model: updated_model,
                callbacks: model.callbacks
                }

        Err _ -> 
            # TODO: Handle this error
            Box.box model

frontend_receive_ws_message_for_host! : Box HostModel, List U8 => Box (HostModel )
frontend_receive_ws_message_for_host! = |boxed, msg_bytes|
    model = (Box.unbox boxed)
    app = InternalFrontend.inner frontendApp
    (updated_frontend_model, m_to_backend_msg) =
        app.decode_to_frontend_msg msg_bytes
            |> app.updateFromBackend
            |> app.update model.frontend_model
    _ = when m_to_backend_msg is 
        Ok to_backend_msg ->
            app.encode_to_backend_msg to_backend_msg
                |> Str.from_utf8_lossy
                |> send_to_backend_impl!
            0
        Err _ -> 0
 
    Box.box {callbacks: model.callbacks, frontend_model: updated_frontend_model}


frontend_host_view! : Box (HostModel ) => { model : Box HostModel, view : List U8 }
frontend_host_view! = |boxed| 
    model = (Box.unbox boxed)
    (encoded, cbs) = (InternalFrontend.inner frontendApp).view model.frontend_model 
        |> InternalView.repr_

    { 
        model: Box.box { 
            frontend_model : model.frontend_model, 
            callbacks: cbs  
        },
        view: encoded
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

