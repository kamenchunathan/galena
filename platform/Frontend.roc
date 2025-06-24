module [frontend, Frontend]

import Internal.Frontend exposing [InternalFrontend, frontend_]

Frontend model msg toFrontendMsg toBackendMsg : InternalFrontend model msg toFrontendMsg toBackendMsg

frontend = frontend_

