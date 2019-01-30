# Rust Simple webserver

Rust-SimpleHTTP | [Website](http://www.banyango.com/rust-simplehttp/) | [![Build Status](https://travis-ci.org/Banyango/rust-simplehttp.svg?branch=master)](https://travis-ci.org/Banyango/rust-simplehttp)

I was inspired by SimpleHTTP server in python to write up a simple static file server in Rust.

This is currently WIP so there's lots missing. But it's capable of listening on a port and serving up simple static content to a browser.

to build run ```cargo run```

```
usage:
    -p [PORT] : Defaults to listening on port 3000 but you can specify another 
                port to listen on.
    -v        : Verbose mode flag, prints extra debug info about request/responses and such.
    -h        : Hot reload mode. Will reload your browser page based on file changes 
                in the server dir. Uses a websocket on port 30012

Server will listen on localhost:3000 by default.

```
