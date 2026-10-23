"""CPU-only real WebSocket/request lifecycle diagnostic; no model or GPU."""
from __future__ import annotations
import asyncio
from collections.abc import Callable
import gc
import json
from pathlib import Path
import socket
import sys
from threading import Event
import time
import tracemalloc

ROOT=Path('D:/Dev/nemu-wt/P4-02')
sys.path[:0]=[str(ROOT/'services/voice-sidecar'),str(ROOT/'services/voice-sidecar/src')]
import uvicorn
from websockets.asyncio.client import connect
from benchmark import rss
from nemu_voice_sidecar.app import create_app
from nemu_voice_sidecar.backend import Request
from nemu_voice_sidecar.voices import Reference, VoiceBank

class ProbeEngine:
    def warmup(self) -> None: pass
    def health(self) -> dict[str, object]: return {'reasons': [], 'modelLoaded': True}
    def stream(self, request: Request, reference: Reference, cancel: Event, emit: Callable[[bytes],None]) -> None:
        for _ in range(64):
            if cancel.is_set(): return
            emit(bytes(3840))

async def main() -> None:
    output=Path('D:/Dev/cache/p4-02/transport-resource-20261023')
    output.mkdir(exist_ok=False)
    with socket.socket() as free:
        free.bind(('127.0.0.1',0)); port=free.getsockname()[1]
    engine=ProbeEngine()
    engine.warmup()
    app=create_app(engine,VoiceBank(ROOT/'services/voice-sidecar/tests/fixtures/voice-design-v1/voice.json'))
    # Exercise normal /warmup and transport routes; no TTS implementation substitute is used for acceptance.
    server=uvicorn.Server(uvicorn.Config(app,host='127.0.0.1',port=port,log_level='warning',access_log=False))
    serving=asyncio.create_task(server.serve())
    while not server.started:
        if serving.done(): await serving; raise RuntimeError('SERVER_NOT_READY')
        await asyncio.sleep(.01)
    import httpx
    async with httpx.AsyncClient(trust_env=False) as client:
        response=await client.post(f'http://127.0.0.1:{port}/warmup')
        assert response.json()['ready']
    async def request() -> None:
        size=0
        async with connect(f'ws://127.0.0.1:{port}/synthesize',proxy=None) as ws:
            await ws.send(json.dumps({'text':'synthetic transport diagnostic','lang':'zh','style':'neutral','voiceId':'p4-02-synthetic-test','sampleRate':24000}))
            async for block in ws:
                assert isinstance(block,bytes)
                size+=len(block)
        assert size==64*3840
    def state() -> dict[str,object]:
        current,peak=tracemalloc.get_traced_memory()
        return {'rss':rss(),'python_current':current,'python_peak':peak,'tracer_metadata':tracemalloc.get_tracemalloc_memory(),'gc_counts':gc.get_count(),'gc_stats':gc.get_stats(),'tasks':len(asyncio.all_tasks())}
    try:
        for _ in range(10): await request()
        gc.collect()
        tracemalloc.start(1)
        before=tracemalloc.take_snapshot()
        initial=state()
        started=time.monotonic()
        with (output/'samples.jsonl').open('x',encoding='utf-8') as raw:
            for index in range(1000):
                await request()
                if index%20==19:
                    row={'requests':index+1,'seconds':time.monotonic()-started,**state()}
                    raw.write(json.dumps(row)+'\n'); raw.flush()
        final=state()
        after=tracemalloc.take_snapshot()
        gc.collect()
        await asyncio.sleep(0)
        post_gc=state()
        result={'acceptance':False,'gpu_used':False,'model_used':False,'requests':1000,'blocks_per_request':64,'seconds':time.monotonic()-started,'initial':initial,'final':final,'post_gc':post_gc,'differences':[{'bytes':s.size_diff,'count':s.count_diff,'location':str(s.traceback)} for s in after.compare_to(before,'lineno')[:25]]}
        (output/'result.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
        print(json.dumps(result,indent=2))
    finally:
        tracemalloc.stop()
        server.should_exit=True
        await serving
asyncio.run(main())
