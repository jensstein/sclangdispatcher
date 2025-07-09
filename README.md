# sclang dispatcher
A server and client for sending commands to the SuperCollider language
server.
This application serves the same purpose as the dispatcher from scvim:
https://github.com/supercollider/scvim/tree/4b738f8a13056e0a74227135ff928c0149d233eb/bin

## Usage
- **server**: Start SuperCollider and the server for listening for
  commands.
```
$ sclangdispatcher server -h
Start sclang server

Usage: sclangdispatcher server [OPTIONS]

Options:
      --host <HOST>            Host address [default: 0.0.0.0]
  -p, --port <PORT>            Port for listening [default: 5000]
  -i, --ide-class <IDE_CLASS>  Name of ide [default: scvim]
  -h, --help                   Print help
```
- **client**: Send commands to the sclang server
```
$ sclangdispatcher client -h
Send command to the sclang server

Usage: sclangdispatcher client [OPTIONS] <COMMAND>

Arguments:
  <COMMAND>  Command to send

Options:
      --host <HOST>  Host address of server [default: http://127.0.0.1]
  -p, --port <PORT>  Port of server [default: 5000]
  -h, --help         Print help
```

## Development
- Run `git config --local core.hooksPath .githooks` to set up the git
hooks.
- Compile with `cargo build --release`
