# platform "galena_platform"
#     requires { FrontendModel, BackendModel, ToFrontendMsg, ToBackendMsg } {
#         frontendApp : Frontend FrontendModel frontendMsg ToFrontendMsg ToBackendMsg,
#         backendApp : Backend BackendModel backendMsg ToFrontendMsg ToBackendMsg,
#     }
#     exposes [Frontend, Backend, Cmd]
#     packages {}
#     imports []
#     provides [mainForHost!]
#
# import Frontend exposing [Frontend]
# import Backend exposing [Backend]
#
# mainForHost! : I32 => I32
# mainForHost! = \_ -> const (frontendApp, backendApp) 0
#
# const = \_, a -> a
platform "galena_platform"
    requires {} {}
    exposes [Frontend, Backend, Cmd]
    packages {}
    imports []
    provides [mainForHost!]

mainForHost! : I32 => I32
mainForHost! = |_| 8492

