from __future__ import annotations
from collections import Counter, defaultdict
from datetime import datetime, timezone
import gzip, hashlib, importlib.metadata, json, math, re, subprocess, sys, time
from pathlib import Path
from typing import Any

REPO=Path(r"D:\Dev\nemu-wt\P0-10")
CACHE=Path(r"D:\Dev\cache\P0-10")
RUN=CACHE/"benchmark-runs/run-03"
PREP=CACHE/"prepared/full-v5"
OUT=CACHE/"benchmark-run03-audit-01"
CARRIER=REPO/"spikes/carrier-engine"
SOURCE=REPO/"spikes/codex-app-server/src"
T0=datetime.now(timezone.utc)
M0=time.monotonic()

def utc(dt=None):
    return (dt or datetime.now(timezone.utc)).isoformat().replace("+00:00","Z")
def sha(path):
    h=hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda:f.read(8*1024*1024),b""): h.update(block)
    return h.hexdigest()
def dump_new(path,obj):
    with path.open("x",encoding="utf-8",newline="\n") as f:
        json.dump(obj,f,ensure_ascii=False,indent=2,allow_nan=False); f.write("\n")
def load(path):
    return json.loads(path.read_text(encoding="utf-8"))
def reject_constant(x):
    raise ValueError("Non-JSON numeric constant: "+x)
def check(checks,failures,q,sid,name,ok,details=None):
    c=checks.setdefault(name,{"rows":0,"matching":0}); c["rows"]+=1
    if ok: c["matching"]+=1
    else: failures.append({"quantization":q,"id":sid,"check":name,"details":details})
def pct(values,p):
    a=sorted(values)
    if not a: return {"nearestRank":None,"numpyStyleLinear":None}
    rank=max(0,math.ceil(p*len(a))-1); pos=(len(a)-1)*p
    lo=math.floor(pos); hi=math.ceil(pos); w=pos-lo
    return {"nearestRank":a[rank],"numpyStyleLinear":a[lo]*(1-w)+a[hi]*w}
def dist(values):
    return {"count":len(values),"p50":pct(values,.5),"p90":pct(values,.9)}
def timing(resp,key):
    x=resp.get("final",{}).get("timings",{}).get(key)
    return float(x) if isinstance(x,(int,float)) and not isinstance(x,bool) else None

def memory(path):
    if not path.is_file(): return {"present":False,"measurement":"not measured","path":str(path)}
    rows=matched=counter_n=0; errors=[]; starts=[]; ds=[]; ss=[]; sums=[]
    pat=re.compile(r"^pid_(\d+)_")
    with path.open(encoding="utf-8") as f:
        for no,line in enumerate(f,1):
            rows+=1
            try:
                x=json.loads(line); st=str(x["startedUtc"])
                starts.append((datetime.fromisoformat(st.replace("Z","+00:00")),st))
                pids={int(p["ProcessId"]) for p in x.get("ownedProcesses",[])
                      if isinstance(p,dict) and isinstance(p.get("ProcessId"),int)}
                d=s=n=0
                for c in x.get("counters",[]) if isinstance(x.get("counters"),list) else []:
                    m=pat.match(str(c.get("Name",""))) if isinstance(c,dict) else None
                    if not m or int(m.group(1)) not in pids: continue
                    if isinstance(c.get("DedicatedUsage"),int): d+=c["DedicatedUsage"]
                    if isinstance(c.get("SharedUsage"),int): s+=c["SharedUsage"]
                    n+=1
                if n:
                    matched+=1; counter_n+=n; ds.append((d,st)); ss.append((s,st)); sums.append((d+s,st))
            except Exception as e: errors.append({"line":no,"error":f"{type(e).__name__}: {e}"})
    starts.sort()
    gaps=[((b[0]-a[0]).total_seconds(),a[1],b[1]) for a,b in zip(starts,starts[1:])]
    md=max(ds,key=lambda z:z[0]) if ds else None
    ms=max(ss,key=lambda z:z[0]) if ss else None
    mc=max(sums,key=lambda z:z[0]) if sums else None
    mg=max(gaps,key=lambda z:z[0]) if gaps else None
    return {"present":True,"path":str(path),"scopeFromFile":"WDDM process counters, sampled; not an exact continuous peak",
      "snapshotRows":rows,"matchedRows":matched,"matchedCounterRecords":counter_n,
      "measurement":"sampled matched process counters" if md and ms else "not measured",
      "maxDedicatedUsageBytes":md[0] if md else None,"maxDedicatedAtUtc":md[1] if md else None,
      "maxSharedUsageBytes":ms[0] if ms else None,"maxSharedAtUtc":ms[1] if ms else None,
      "sumOfSeparateMaximaBytes":md[0]+ms[0] if md and ms else None,
      "maxPerSnapshotDedicatedPlusSharedBytes":mc[0] if mc else None,"maxCombinedAtUtc":mc[1] if mc else None,
      "maximumSampleStartIntervalSeconds":mg[0] if mg else None,
      "maximumSampleStartIntervalFromUtc":mg[1] if mg else None,"maximumSampleStartIntervalToUtc":mg[2] if mg else None,
      "parseErrors":errors}

def main():
    manifest=[]
    for group,root in (("run-03",RUN),("prepared-full-v5",PREP)):
        for p in sorted((x for x in root.rglob("*") if x.is_file()),key=lambda x:x.as_posix()):
            manifest.append({"group":group,"path":p.relative_to(root).as_posix(),
              "bytes":p.stat().st_size,"sha256":sha(p)})
    sources=[REPO/"AGENTS.md",REPO/".codex/agents/routine-reporter.toml",
      REPO/"spikes/codex-app-server/fixtures/irl-cases.json",
      REPO/"spikes/codex-app-server/semantic-candidate.schema.json",
      REPO/"spikes/codex-app-server/semantic-candidate.wire.schema.json",
      SOURCE/"local_irl.py",SOURCE/"decoder_adapter.py",CARRIER/"compact_codec.py",
      CARRIER/"schema_validation.py",CARRIER/"benchmark_native.py",
      CARRIER/"benchmark_plan.py",CARRIER/"http_probe.py"]
    for p in sources:
        manifest.append({"group":"repository-source","path":p.relative_to(REPO).as_posix(),
          **({"bytes":p.stat().st_size,"sha256":sha(p)} if p.is_file() else {"missing":True})})
    head=subprocess.run(["git","rev-parse","HEAD"],cwd=REPO,capture_output=True,text=True,encoding="utf-8",check=True).stdout.strip()
    script=OUT/"audit_run03.py"
    hashes={"createdUtc":utc(),"workingDirectory":str(REPO),"gitHead":head,
      "methodScript":{"path":str(script),"bytes":script.stat().st_size,"sha256":sha(script)},
      "python":sys.version,"jsonschemaVersion":importlib.metadata.version("jsonschema"),
      "entries":manifest,"entryCount":len(manifest),"totalInputBytes":sum(x.get("bytes",0) for x in manifest)}
    dump_new(OUT/"input-hashes.json",hashes)
    with gzip.open(PREP/"jobs.json.gz","rt",encoding="utf-8") as f: jobs=json.load(f)
    plan=load(PREP/"plan.json"); schemas=load(PREP/"schemas.json")
    full_schema=load(REPO/"spikes/codex-app-server/semantic-candidate.schema.json")
    wire_schema=load(REPO/"spikes/codex-app-server/semantic-candidate.wire.schema.json")
    sys.path.insert(0,str(CARRIER)); sys.path.insert(0,str(SOURCE))
    import compact_codec as codec
    import schema_validation
    import local_irl
    import benchmark_plan
    cv=schema_validation.validator(schemas["compact"])
    fv=schema_validation.validator(full_schema)
    wv=schema_validation.validator(wire_schema)
    compact_schema_matches=codec.compact_schema()==schemas["compact"]
    fixed=benchmark_plan.FIXED_SENTENCE
    context=plan.get("context")
    expected={"speech":{"warmup":1,"formal":20},"unconstrained":{"warmup":1,"formal":50},
              "wire":{"warmup":1,"formal":200},"compact":{"warmup":1,"formal":200}}
    audit={"method":"Independent offline audit of run-03 raw JSON, request/response/SSE, schemas and sampled counters.",
      "startedUtc":utc(T0),"workingDirectory":str(REPO),"gitHead":head,
      "inputHashesPath":str(OUT/"input-hashes.json"),"gpuModelCompileOrTestStarted":False,
      "compactSchemaGeneratedMatchesPrepared":compact_schema_matches,
      "httpCompletionEvidence":"http_probe.py writes response.json after urlopen iterator and context exit; check response, event lines/count and final stop. Raw HTTP status is not persisted.",
      "quantizations":{}}
    all_transport=[]; all_candidates=[]; all_candidate_mismatches=[]; all_invalid=[]; all_stream_mismatches=[]

    for q in ("Q4_K_M","Q8_0"):
        root=RUN/q
        rows=[json.loads(s) for s in (root/"samples.jsonl").read_text(encoding="utf-8").splitlines() if s]
        rowmap={str(x["id"]):x for x in rows}
        checks={}; failures=[]
        order=[str(x.get("id")) for x in rows]==[str(x.get("id")) for x in jobs]
        counts={arm:{"warmup":sum(x.get("arm")==arm and x.get("warmup") is True for x in rows),
                     "formal":sum(x.get("arm")==arm and x.get("warmup") is False for x in rows)}
                for arm in expected}
        transport={}; token_records=[]
        for ji,job in enumerate(jobs):
            sid=str(job["id"]); row=rowmap.get(sid)
            if row is None:
                check(checks,failures,q,sid,"sample.rowExists",False); continue
            check(checks,failures,q,sid,"sample.idMatchesPrepared",row.get("id")==job.get("id"))
            check(checks,failures,q,sid,"sample.armMatchesPrepared",row.get("arm")==job.get("arm"),{"row":row.get("arm"),"prepared":job.get("arm")})
            check(checks,failures,q,sid,"sample.warmupMatchesPrepared",row.get("warmup") is job.get("warmup"))
            check(checks,failures,q,sid,"sample.payloadMatchesPrepared",row.get("sample")==job.get("sample"))
            ph=hashlib.sha256(str(job.get("prompt","")).encode("utf-8")).hexdigest()
            check(checks,failures,q,sid,"sample.promptHashMatchesText",job.get("promptSha256")==ph,{"prepared":job.get("promptSha256"),"calculated":ph})
            check(checks,failures,q,sid,"sample.promptHashMatchesRow",row.get("promptSha256")==job.get("promptSha256"))
            ids=job.get("inputTokenIds")
            check(checks,failures,q,sid,"sample.inputTokenCountMatchesPrepared",row.get("inputTokens")==len(ids) if isinstance(ids,list) else False)
            check(checks,failures,q,sid,"sample.orderMatchesPrepared",order,{"rowIndex":rows.index(row),"preparedIndex":ji})
            reqrel=f"requests/{sid}/request.json"; resrel=f"requests/{sid}/response.json"
            check(checks,failures,q,sid,"sample.requestPathMatchesId",row.get("rawRequest")==reqrel,row.get("rawRequest"))
            check(checks,failures,q,sid,"sample.responsePathMatchesId",row.get("rawResponse")==resrel,row.get("rawResponse"))
            reqp=root/str(row.get("rawRequest","")); resp=root/str(row.get("rawResponse",""))
            req={}; response={}; events=[]
            if reqp.is_file():
                try:
                    x=load(reqp); req=x if isinstance(x,dict) else {}
                    check(checks,failures,q,sid,"request.jsonParses",isinstance(x,dict),type(x).__name__)
                except Exception as e: check(checks,failures,q,sid,"request.jsonParses",False,f"{type(e).__name__}: {e}")
            else: check(checks,failures,q,sid,"request.fileExists",False,str(reqp))
            if resp.is_file():
                try:
                    x=load(resp); response=x if isinstance(x,dict) else {}
                    check(checks,failures,q,sid,"response.jsonParses",isinstance(x,dict),type(x).__name__)
                except Exception as e: check(checks,failures,q,sid,"response.jsonParses",False,f"{type(e).__name__}: {e}")
            else: check(checks,failures,q,sid,"response.fileExists",False,str(resp))
            ep=resp.parent/"events.jsonl"
            if ep.is_file():
                for ln,line in enumerate(ep.read_text(encoding="utf-8").splitlines(),1):
                    if not line: continue
                    try:
                        x=json.loads(line)
                        if isinstance(x,dict): events.append(x)
                        else: check(checks,failures,q,sid,"events.lineIsObject",False,{"line":ln,"type":type(x).__name__})
                    except Exception as e: check(checks,failures,q,sid,"events.lineParses",False,{"line":ln,"error":f"{type(e).__name__}: {e}"})
            else: check(checks,failures,q,sid,"events.fileExists",False,str(ep))
            failp=resp.parent/"failure.json"
            check(checks,failures,q,sid,"http.failureFileAbsent",not failp.exists(),str(failp) if failp.exists() else None)
            prompt=req.get("prompt")
            check(checks,failures,q,sid,"request.promptIdsMatchPrepared",prompt==ids,{"request":len(prompt) if isinstance(prompt,list) else None,"prepared":len(ids) if isinstance(ids,list) else None})
            check(checks,failures,q,sid,"request.nPredictMatchesPrepared",req.get("n_predict")==job.get("maxNewTokens"))
            check(checks,failures,q,sid,"request.streamingAndTokenIdsEnabled",req.get("stream") is True and req.get("return_tokens") is True)
            check(checks,failures,q,sid,"request.promptCacheDisabled",req.get("cache_prompt") is False)
            contextfits=isinstance(ids,list) and isinstance(job.get("maxNewTokens"),int) and isinstance(context,int) and len(ids)+job["maxNewTokens"]<=context
            check(checks,failures,q,sid,"prepared.contextNotExceeded",contextfits,{"prompt":len(ids) if isinstance(ids,list) else None,"maxNewTokens":job.get("maxNewTokens"),"context":context})
            final=response.get("final"); final=final if isinstance(final,dict) else {}
            tm=final.get("timings"); tm=tm if isinstance(tm,dict) else {}
            check(checks,failures,q,sid,"response.eventCountMatchesLines",response.get("eventCount")==len(events),{"response":response.get("eventCount"),"lines":len(events)})
            last=events[-1].get("data") if events and isinstance(events[-1],dict) else None
            check(checks,failures,q,sid,"response.finalSseStopTrue",isinstance(last,dict) and last.get("stop") is True,last.get("stop") if isinstance(last,dict) else None)
            check(checks,failures,q,sid,"response.finalMatchesLastEvent",isinstance(last,dict) and final==last)
            text="".join(e.get("data",{}).get("content","") for e in events if isinstance(e.get("data"),dict) and isinstance(e["data"].get("content",""),str))
            check(checks,failures,q,sid,"response.contentReassemblesFromSse",text==response.get("content"),{"reconstructedChars":len(text),"responseChars":len(str(response.get("content","")))})
            check(checks,failures,q,sid,"response.cacheNZero",tm.get("cache_n")==0,tm.get("cache_n"))
            check(checks,failures,q,sid,"response.promptNMatchesInput",tm.get("prompt_n")==len(prompt) if isinstance(prompt,list) else False,{"promptN":tm.get("prompt_n"),"input":len(prompt) if isinstance(prompt,list) else None})
            check(checks,failures,q,sid,"response.truncatedFalse",final.get("truncated") is False,final.get("truncated"))
            check(checks,failures,q,sid,"sample.timingsEqualFinal",row.get("timings")==tm)
            for k in ("ttftClientMs","completedClientMs","firstSentenceClientMs"):
                check(checks,failures,q,sid,"sample."+k+"MatchesResponse",row.get(k)==response.get(k),{"sample":row.get(k),"response":response.get(k)})
            check(checks,failures,q,sid,"sample.stopTypeMatchesFinal",row.get("stopType")==final.get("stop_type"))
            check(checks,failures,q,sid,"native.finalTokensPredictedMatchesTiming",final.get("tokens_predicted")==tm.get("predicted_n"))
            times=response.get("clientTokenTimesMs"); stream_n=len(times) if isinstance(times,list) else 0
            sse_lens=[]; event_pred=[]; bad_event_indexes=[]
            for ei,event in enumerate(events):
                data=event.get("data",{})
                if not isinstance(data,dict): bad_event_indexes.append(ei); continue
                toks=data.get("tokens",[])
                if isinstance(toks,list): sse_lens.append(len(toks))
                else: bad_event_indexes.append(ei)
                if "tokens_predicted" in data: event_pred.append(data["tokens_predicted"])
            sse_n=sum(sse_lens)
            check(checks,failures,q,sid,"response.clientTokenTimesMatchSseTokenArrays",stream_n==sse_n,{"times":stream_n,"SSE":sse_n})
            check(checks,failures,q,sid,"sample.returnedTokenCountMatchesResponse",row.get("returnedStreamTokenCount")==stream_n)
            rec={"quantization":q,"id":sid,"arm":row.get("arm"),
              "lang":row.get("sample",{}).get("lang") if isinstance(row.get("sample"),dict) else None,
              "returnedStreamTokenCountFromResponse":stream_n,"returnedStreamTokenCountFromSampleRow":row.get("returnedStreamTokenCount"),
              "sseEventTokenArrayTotal":sse_n,"nativeTimingsPredictedN":tm.get("predicted_n"),
              "finalEventTokensPredicted":final.get("tokens_predicted"),
              "eventTokensPredictedSequence":event_pred,"sseEventTokenArrayLengths":sse_lens,
              "finalSseEventStop":last.get("stop") if isinstance(last,dict) else None,
              "finalSseEventTokenArray":last.get("tokens") if isinstance(last,dict) else None,
              "streamMinusNativeCount":stream_n-tm.get("predicted_n") if isinstance(tm.get("predicted_n"),int) else None,
              "malformedEventDataIndexes":bad_event_indexes}
            transport[sid]={"job":job,"row":row,"request":req,"response":response,"events":events,"stream":rec}
            token_records.append(rec)
            if rec["streamMinusNativeCount"] not in (None,0): all_stream_mismatches.append(rec)
        candidates=[]; mismatches=[]; invalid=[]
        by_lang=defaultdict(lambda:defaultdict(lambda:{
          "rows":0,"independentSchemaValidTrue":0,"independentSchemaValidFalse":0,
          "sampleSchemaValidTrue":0,"sampleSchemaValidFalse":0,
          "independentSourceBindingValidTrue":0,"independentSourceBindingValidFalse":0,
          "sampleSourceBindingValidTrue":0,"sampleSourceBindingValidFalse":0,
          "booleanComparisonMismatches":0,"wireSchemaValidTrue":0,"wireSchemaValidFalse":0,
          "compactSchemaValidTrue":0,"compactSchemaValidFalse":0}))
        for job in jobs:
            arm=str(job["arm"])
            if arm=="speech": continue
            sid=str(job["id"]); tr=transport.get(sid)
            if tr is None: continue
            row=tr["row"]; response=tr["response"]; sample=job.get("sample")
            lang=str(sample.get("lang")) if isinstance(sample,dict) else "missing"
            content=response.get("content")
            parse_error=compact_error=wire_error=full_error=source_error=None
            candidate=None; compact_valid=None; wire_valid=None; full_valid=False; source_valid=False
            try:
                if not isinstance(content,str): raise TypeError("response.content is not a string")
                candidate=json.loads(content,parse_constant=reject_constant)
            except Exception as e: parse_error=f"{type(e).__name__}: {e}"
            if parse_error is None:
                if arm=="compact":
                    try: cv.validate(candidate); compact_valid=True
                    except Exception as e:
                        compact_valid=False; compact_error=f"{type(e).__name__}: {e}"
                    if compact_valid:
                        try:
                            if not isinstance(candidate,dict): raise ValueError("Compact root must be an object")
                            candidate=codec.expand(candidate)
                        except Exception as e:
                            compact_error=f"{type(e).__name__}: {e}"; full_error=compact_error
                if arm=="wire":
                    try: wv.validate(candidate); wire_valid=True
                    except Exception as e:
                        wire_valid=False; wire_error=f"{type(e).__name__}: {e}"
                if arm!="compact" or compact_valid is True:
                    try: fv.validate(candidate); full_valid=True
                    except Exception as e: full_error=f"{type(e).__name__}: {e}"
            if full_valid:
                try: source_valid=bool(local_irl.source_valid(candidate,sample))
                except Exception as e:
                    source_error=f"{type(e).__name__}: {e}"; source_valid=False
            row_schema=row.get("schemaValid"); row_source=row.get("sourceBindingValid")
            schema_same=row_schema is full_valid; source_same=row_source is source_valid
            group=by_lang[arm][lang]; group["rows"]+=1
            group["independentSchemaValidTrue" if full_valid else "independentSchemaValidFalse"]+=1
            if row_schema is True: group["sampleSchemaValidTrue"]+=1
            elif row_schema is False: group["sampleSchemaValidFalse"]+=1
            group["independentSourceBindingValidTrue" if source_valid else "independentSourceBindingValidFalse"]+=1
            if row_source is True: group["sampleSourceBindingValidTrue"]+=1
            elif row_source is False: group["sampleSourceBindingValidFalse"]+=1
            if not(schema_same and source_same): group["booleanComparisonMismatches"]+=1
            if wire_valid is not None: group["wireSchemaValidTrue" if wire_valid else "wireSchemaValidFalse"]+=1
            if compact_valid is not None: group["compactSchemaValidTrue" if compact_valid else "compactSchemaValidFalse"]+=1
            result={"quantization":q,"id":sid,"arm":arm,"lang":lang,
              "independentParseValid":parse_error is None,"parseError":parse_error,
              "independentCompactSchemaValid":compact_valid,"compactError":compact_error,
              "independentWireSchemaValid":wire_valid,"wireError":wire_error,
              "independentFullSchemaValid":full_valid,"fullSchemaError":full_error,
              "independentSourceBindingValid":source_valid,"sourceValidationError":source_error,
              "sampleRowSchemaValid":row_schema,"sampleRowSourceBindingValid":row_source,
              "schemaBooleanMatchesSample":schema_same,"sourceBooleanMatchesSample":source_same,
              "bothBooleansMatchSample":schema_same and source_same,
              "sampleRowValidationError":row.get("validationError")}
            candidates.append(result); all_candidates.append(result)
            if not(schema_same and source_same):
                mismatches.append(result); all_candidate_mismatches.append(result)
            if not full_valid or not source_valid:
                invalid.append(result); all_invalid.append(result)
        candidate_report={"responseContentReparsed":True,"formalCandidateRows":len(candidates),
          "arms":{arm:{lang:vals for lang,vals in sorted(langs.items())} for arm,langs in sorted(by_lang.items())},
          "candidateBooleanMismatchCount":len(mismatches),"candidateBooleanMismatches":mismatches,
          "invalidCandidateCount":len(invalid),"invalidCandidateRows":invalid,"allCandidateResults":candidates}

        speech_rows=[]; speech_freq=Counter()
        for job in jobs:
            if job.get("arm")!="speech" or job.get("warmup") is not False: continue
            sid=str(job["id"]); tr=transport.get(sid)
            if tr is None: continue
            out=tr["response"].get("content")
            out=out if isinstance(out,str) else str(out)
            speech_freq[out]+=1
            speech_rows.append({"id":sid,"exactFixedSentenceMatch":out==fixed,"output":out})
        speech={"fixedSentence":fixed,"formalRows":len(speech_rows),
          "exactMatchCount":sum(x["exactFixedSentenceMatch"] for x in speech_rows),
          "outputFrequency":[{"output":s,"count":n} for s,n in sorted(speech_freq.items(),key=lambda z:(-z[1],z[0]))],
          "rows":speech_rows}
        first50={}
        for arm in ("wire","unconstrained"):
            chosen=[j for j in jobs if j.get("arm")==arm and j.get("warmup") is False
                    and isinstance(j.get("sample"),dict)][:50]
            ids50=[str(j["id"]) for j in chosen]
            def vals(key):
                out=[]
                for sid in ids50:
                    tr=transport.get(sid)
                    if tr:
                        x=timing(tr["response"],key)
                        if x is not None: out.append(x)
                return out
            ttft=[float(transport[sid]["response"]["ttftClientMs"]) for sid in ids50
                  if sid in transport and isinstance(transport[sid]["response"].get("ttftClientMs"),(int,float))
                  and not isinstance(transport[sid]["response"].get("ttftClientMs"),bool)]
            complete=[float(transport[sid]["response"]["completedClientMs"]) for sid in ids50
                      if sid in transport and isinstance(transport[sid]["response"].get("completedClientMs"),(int,float))
                      and not isinstance(transport[sid]["response"].get("completedClientMs"),bool)]
            calculated=[]; compared=matched=0
            for sid in ids50:
                tr=transport.get(sid)
                if not tr: continue
                tm=tr["response"].get("final",{}).get("timings",{})
                n=tm.get("predicted_n"); ms=tm.get("predicted_ms"); stored=tm.get("predicted_per_second")
                if isinstance(n,(int,float)) and isinstance(ms,(int,float)) and not isinstance(ms,bool) and ms>0:
                    rate=float(n)*1000.0/float(ms); calculated.append(rate)
                    if isinstance(stored,(int,float)) and not isinstance(stored,bool):
                        compared+=1
                        if math.isclose(rate,float(stored),rel_tol=1e-12,abs_tol=1e-12): matched+=1
            first50[arm]={"selectedRows":len(ids50),
              "sampleIdsInPreparedOrder":[str(j["sample"]["id"]) for j in chosen],
              "nativePromptMs":dist(vals("prompt_ms")),"clientTTFTMs":dist(ttft),
              "clientCompleteMs":dist(complete),"nativeDecodeTPSFromTimings":dist(vals("predicted_per_second")),
              "nativeDecodeTPSRecalculatedFromPredictedNAndMs":dist(calculated),
              "decodeTPSFormulaComparison":{"compared":compared,"closeMatches":matched},
              "ttftCoverage":len(ttft)}
        output_pct={}
        for arm in ("wire","compact"):
            chosen=[j for j in jobs if j.get("arm")==arm and j.get("warmup") is False]
            stream=[]; native=[]; complete=[]
            for j in chosen:
                tr=transport.get(str(j["id"]))
                if not tr: continue
                sn=tr["stream"]["returnedStreamTokenCountFromResponse"]
                nn=tr["stream"]["nativeTimingsPredictedN"]
                cn=tr["response"].get("completedClientMs")
                if isinstance(sn,int) and not isinstance(sn,bool): stream.append(float(sn))
                if isinstance(nn,int) and not isinstance(nn,bool): native.append(float(nn))
                if isinstance(cn,(int,float)) and not isinstance(cn,bool): complete.append(float(cn))
            output_pct[arm]={"formalRows":len(chosen),
              "streamReturnedTokenCount":dist(stream),"nativePredictedTokenCount":dist(native),
              "clientCompleteMs":dist(complete)}
        token_diffs=[x for x in token_records if isinstance(x.get("streamMinusNativeCount"),int)
                     and x["streamMinusNativeCount"]!=0]
        report={"sampleInventory":{"rowCount":len(rows),"preparedJobCount":len(jobs),
                  "preparedOrderMatchesRows":order,"armCounts":counts,
                  "armCountsMatchPlan":counts==expected,"expectedArmCounts":expected,
                  "promptContextLimit":context},
          "transport":{"checks":checks,"failures":failures,"failureCount":len(failures),
            "httpCompletionEvidence":"response.json exists and http_probe.py writes only after urlopen read loop/context exit; event file, count and terminal stop checked; HTTP status is not persisted."},
          "streamVsNativeTokenCounts":{
            "rowsWithNativeCount":sum(isinstance(x.get("nativeTimingsPredictedN"),int) for x in token_records),
            "streamNativeMismatchCount":len(token_diffs),
            "streamSseArrayMismatchCount":sum(x["returnedStreamTokenCountFromResponse"]!=x["sseEventTokenArrayTotal"] for x in token_records),
            "mismatches":token_diffs},
          "irlRevalidation":candidate_report,"speech":speech,"first50Metrics":first50,
          "formal200OutputAndCompletionPercentiles":output_pct,"memoryCounters":memory(root/"server/memory.jsonl")}
        audit["quantizations"][q]=report
        all_transport.extend(failures)

    audit["globalCounts"]={"transportFailureCount":len(all_transport),
      "candidateRowsRevalidated":len(all_candidates),
      "candidateBooleanMismatchCount":len(all_candidate_mismatches),
      "invalidCandidateRowCount":len(all_invalid),
      "streamNativeCountMismatchCount":len(all_stream_mismatches),
      "streamNativeCountMismatchIds":[{"quantization":x["quantization"],"id":x["id"],
        "arm":x["arm"],"stream":x["returnedStreamTokenCountFromResponse"],
        "native":x["nativeTimingsPredictedN"],
        "eventTokensPredicted":x["eventTokensPredictedSequence"]} for x in all_stream_mismatches],
      "allCandidateBooleanMismatches":all_candidate_mismatches,
      "allInvalidCandidateRows":all_invalid,"allTransportFailures":all_transport}
    audit["endedUtc"]=utc(); audit["elapsedSeconds"]=time.monotonic()-M0
    audit["auditScriptCompleted"]=True
    dump_new(OUT/"audit.json",audit)
    print(json.dumps({"auditScriptCompleted":True,"gitHead":head,
      "quantizations":{q:{"rows":v["sampleInventory"]["rowCount"],
        "transportFailures":v["transport"]["failureCount"],
        "irlRows":v["irlRevalidation"]["formalCandidateRows"],
        "candidateBooleanMismatches":v["irlRevalidation"]["candidateBooleanMismatchCount"],
        "streamNativeCountMismatches":v["streamVsNativeTokenCounts"]["streamNativeMismatchCount"],
        "memoryRows":v["memoryCounters"].get("snapshotRows"),
        "memoryMatchedRows":v["memoryCounters"].get("matchedRows")} for q,v in audit["quantizations"].items()},
      "elapsedSeconds":audit["elapsedSeconds"]},ensure_ascii=False))

if __name__=="__main__":
    main()
