module [Backend, backend]

import BackendInternal exposing [BackendInternal, backend_]

Backend model msg toFrontendMsg toBackendMsg : BackendInternal model msg toFrontendMsg toBackendMsg

backend = backend_
