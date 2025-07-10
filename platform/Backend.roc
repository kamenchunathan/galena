module [Backend, backend]

import Internal.Backend exposing [BackendInternal, backend_]

Backend model msg toFrontendMsg toBackendMsg : BackendInternal model msg toFrontendMsg toBackendMsg

backend = backend_
