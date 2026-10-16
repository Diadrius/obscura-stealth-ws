"""Print the raw WebSocket upgrade request a client sends (header order and casing included).

  python repro/handshake_dump.py 9781
  # then, from page JS in the browser under test: new WebSocket('ws://127.0.0.1:9781')

Run it once with real Chrome and once per Obscura build, then diff the output.
"""
import socket, sys

s = socket.socket()
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(("127.0.0.1", int(sys.argv[1]) if len(sys.argv) > 1 else 9781))
s.listen(5)
while True:
    c, _ = s.accept()
    data = c.recv(8192)
    print(data.decode(errors="replace").replace("\r\n", "\n"), "-" * 40, sep="\n", flush=True)
    c.close()
