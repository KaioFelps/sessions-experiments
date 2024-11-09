# Experimenting Sessions and Flash Messages

Sessions are data stored server-side associated with an Session Id. This application manages it with a handmade
Session Middleware.

## Flow

1. The `SessionMiddleware` captures the session id or generate one if it doesn't exist yet.
2. Using the session id, it gets the associated Session (removing it from the `Sessions` singletone**!**).
3. An `Session` object is stored in the `Request` extensions to be retrieved by the handlers.

## Functionalities

Any `serde_json::Value` object can be stored in a Session. It will persist until the next request of the associated SessionId
(when it gets fetched and, thus, cleaned).

By calling `Sessions::forward` and passing the session as a parameter, it will be available at the next request.
What it does is to replace any existing session by the given one.
