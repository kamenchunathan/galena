app [main!] {
    cli: platform "https://github.com/roc-lang/basic-cli/releases/download/0.19.0/Hj-J_zxz7V9YurCSTFcFdu6cQJie4guzsPMUi5kBYUk.tar.br",
    json: "https://github.com/lukewilliamboswell/roc-json/releases/download/0.12.0/1trwx8sltQ-e9Y2rOB4LWUWLS_sFVyETK8Twl0i9qpw.tar.gz",
}

import cli.Stdout
import json.Json

main! = |_|
    Stdout.line! (Inspect.to_str bar)

BackendInternal model msg toFrontendMsg toBackendMsg := {
    init! : model,
    update! : msg, model => model,
    update_from_frontend : Str, Str, List U8 -> I32,
}

foo : { init! : model, update! : msg, model => model, update_from_frontend : Str, Str, toBackendMsg -> I32 } -> BackendInternal model msg toFrontendMsg toBackendMsg where toBackendMsg implements Decoding
foo = |orig|
    @BackendInternal {
        init!: orig.init!,
        update!: orig.update!,
        update_from_frontend: |client_id, session_id, bytes|
            decoded =
                when Decode.from_bytes bytes Json.utf8 is
                    Ok msg ->
                        msg

                    Err _ ->
                        crash "Unable to decode toBackendMsg"
            orig.update_from_frontend client_id session_id decoded,
    }

bar : BackendInternal I32 _ _ {}
bar = foo {
    init!: 0,
    update!: |_, model| model,
    update_from_frontend: baz,
}

baz : Str, Str, toBackendMsg -> I32
baz = |_, _, _| 0
