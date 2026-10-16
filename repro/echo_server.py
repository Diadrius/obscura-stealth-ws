"""Local WebSocket echo server: python repro/echo_server.py [port]  (needs `pip install websockets`).

Compression is off on purpose: the client advertises permessage-deflate in the handshake but cannot
compress or decompress frames, so a server that accepts it breaks the connection.
"""
import asyncio, sys
import websockets

async def echo(ws):
    async for msg in ws:
        await ws.send(msg)

async def main(port):
    async with websockets.serve(echo, "127.0.0.1", port, compression=None):
        print(f"echo on ws://127.0.0.1:{port}", flush=True)
        await asyncio.Future()

asyncio.run(main(int(sys.argv[1]) if len(sys.argv) > 1 else 9780))
