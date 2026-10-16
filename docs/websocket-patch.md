# WebSocket patch (stealth build, based on v0.2.4)

Two commits on top of the `v0.2.4` tag. Each fixes one problem and can be reviewed (or sent upstream) on its own.

## 1. A real `WebSocket`

In v0.2.4 `new WebSocket(url)` creates an object that reports `open`, silently drops `send()` and never delivers a message.

Reproduce (stock `-stealth` build):

```
obscura serve --port 9555 --stealth --allow-private-network &
python repro/echo_server.py &
python repro/ws_check.py ws://127.0.0.1:9780     # stock: ['open', 'timeout']
```

With the patch: `['open', 'msg:hello', 'close 1000']`.

The socket needs `--stealth`; without it the page gets `error` and `close 1006` instead of a fake `open`. The URL goes through the same private-network check as `fetch` (loopback needs `--allow-private-network`).

Changes: `obscura-net` (`connect_websocket` on the wreq client with its `ws` feature, a pump task, `WsIn`/`WsOut` types), `obscura-js` (`op_ws_*` ops and a `WebSocket` class in `bootstrap.js`). Also: leading whitespace in URLs is trimmed like browsers do, and cookies for `wss://` are matched against the `https://` equivalent (the jar only treated `https` as secure, so `Secure` cookies were never sent on the upgrade).

## 2. Chrome-shaped handshake

The upgrade request is compared by bot detection against what Chrome sends. Differences fixed:

| | Before | After (matches Chrome) |
|---|---|---|
| Header names / order | lowercase, `Host` last | `Host, Connection, Pragma, Cache-Control, User-Agent, Upgrade, Origin, Sec-WebSocket-Version, Accept-Encoding, Accept-Language, Cookie, Sec-WebSocket-Key, Sec-WebSocket-Extensions`, Title-Case |
| `sec-fetch-*` | sent | not sent |
| `Sec-WebSocket-Extensions` | always sent | only with `OBSCURA_WS_ADVERTISE_DEFLATE=1` |
| ClientHello | page profile (h2 + ALPS extension 17613) | ALPN `http/1.1` only, no ALPS |

Reproduce the header difference with `repro/handshake_dump.py`: run it, open `new WebSocket('ws://127.0.0.1:9781')` from real Chrome and from each build, diff the output.

`permessage-deflate` is only offered when `OBSCURA_WS_ADVERTISE_DEFLATE=1` (default off). The underlying library cannot compress or decompress frames, so a server that accepts the offer breaks the connection. Enable it only if a site rejects handshakes without the header and you know its server will not compress. Known remaining difference: `Connection: upgrade` is lowercase here, `Upgrade` in Chrome.

## Building

`.github/workflows/release.yml` builds the Linux x86_64 no-render stealth binary and attaches `obscura-x86_64-linux-stealth.tar.gz` (+ sha256) to a GitHub release when a `v*` tag is pushed. Local build needs cmake, clang/libclang and a C++ compiler (BoringSSL); first build is about 25 minutes on 2 cores:

```
cargo build --release -p obscura-cli --bins --no-default-features --features stealth
```
