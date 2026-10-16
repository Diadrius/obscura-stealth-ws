"""Open a WebSocket from page JS in a running `obscura serve` and report what happened.

  obscura serve --port 9555 --stealth --allow-private-network &   # stealth build, flag needed for 127.0.0.1
  python repro/echo_server.py &     # or any ws:// / wss:// URL
  python repro/ws_check.py ws://127.0.0.1:9780

Stock build: 'open' is logged, then 'timeout' (the echo never arrives).
Patched build: 'open', 'msg:hello', 'close 1000'.
Needs `pip install playwright` (client only, no browser download) and a page to run on.
"""
import asyncio, sys
from playwright.async_api import async_playwright

JS = """(u)=>new Promise(res=>{const log=[];const ws=new WebSocket(u);
  ws.onopen=()=>{log.push('open');ws.send('hello')};
  ws.onmessage=e=>{log.push('msg:'+e.data);ws.close(1000,'bye')};
  ws.onerror=()=>log.push('error');
  ws.onclose=e=>{log.push('close '+e.code);res(log)};
  setTimeout(()=>res(log.concat('timeout')),10000)})"""

async def main(url, cdp="http://127.0.0.1:9555"):
    async with async_playwright() as p:
        b = await p.chromium.connect_over_cdp(cdp)
        ctx = b.contexts[0] if b.contexts else await b.new_context()
        pg = await ctx.new_page()
        await pg.goto("data:text/html,<title>ws</title>")
        print(url, await pg.evaluate(JS, url))

asyncio.run(main(sys.argv[1]))
