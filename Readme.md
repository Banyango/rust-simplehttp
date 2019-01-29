# Rust Simple webserver

I was inspired by SimpleHTTP server in python to write up a simple static file server in Rust.

Currently it's capable of listening on a port and serving up html files from the file system.

to build run ```cargo run```

```
usage:
    -p [PORT] : defaults to listening on port 3000 but you can specify another port to listen on.
    -v [VERBOSE] : print extra debug info about request/responses and such.


Server will listen on localhost:3000

```
