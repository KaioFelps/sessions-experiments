# Flash messages

These are two tries of coding session-based flash messages (data that are only available to the very next request).

# Handmade version
A small stateful session HashMap that holds inner hashmaps (each representing one session). A session key is a
very simple UUID (and thus not safe). Every SessionMap is cleaned once retrieved, and it's retrivied on every request
by the `SessionMiddleware` (that consumes it and inject it to the request extensions as an `Session` instance).

# Actix Session
Actix Session provides a simple API for managing sessions and two simple providers (redis and cookies).
I've implemented a very simple stateful `SessionStore` manager.

Data will persist until the server is down (and the hashmap holding all the sessions is dropped). Data
from keys `"flash"` and `"errors"` are removed from the sessions every request and added as an extension behind the
`OnceSession` struct.

It's missing a way of periodically deleting sessions after the given *Time To Live* duration.

It might be extracted as an small next-request-scoped sessions library built on top of `actix-session`.
