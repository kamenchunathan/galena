module [frontend, Frontend]

import InternalFrontend exposing [InternalFrontend, frontend_]

Frontend model msg toFrontendMsg toBackendMsg : InternalFrontend model msg toFrontendMsg toBackendMsg

frontend = frontend_

