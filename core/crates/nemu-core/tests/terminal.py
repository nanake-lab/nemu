"""Exercise the real rehearsal process with an isolated LOCALAPPDATA and console."""
from __future__ import annotations

import ctypes
import hashlib
from ctypes import wintypes
import json
import msvcrt
import os
from pathlib import Path
import sqlite3
import struct
import subprocess
import sys
import tempfile
from threading import Timer
import time
from contextlib import closing, contextmanager

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "workers/irl/generated"))
from echo import ingress, runtime as codec
from echo.bridge import from_json
from echo.types import AuthStatus, EpistemicType, EventKind, IngressKind as Kind, IngressReason as Reason, UploadStatus, ReceiptStage

LIMITS = codec.Limits(4 << 20, 4096, 64)
STOP_SECONDS = 5
KERNEL = ctypes.WinDLL("kernel32", use_last_error=True)


def powershell(command: str, cwd: Path) -> bytes:
    return subprocess.check_output(
        ["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", command],
        cwd=cwd, timeout=10, creationflags=subprocess.CREATE_NO_WINDOW,
    )


def stop_private_console(pid: int) -> None:
    # Invoked only by a short-lived helper. The host owns a NEW console.
    KERNEL.FreeConsole()
    if not KERNEL.AttachConsole(pid) or not KERNEL.SetConsoleCtrlHandler(None, True):
        raise ctypes.WinError(ctypes.get_last_error())
    if not KERNEL.GenerateConsoleCtrlEvent(0, 0):
        raise ctypes.WinError(ctypes.get_last_error())


@contextmanager
def running(binary: str, root: Path):
    startup = subprocess.STARTUPINFO()
    startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
    startup.wShowWindow = subprocess.SW_HIDE
    child = subprocess.Popen(
        [binary, "--lineage", "rehearsal", "--log-level", "info"],
        env=os.environ | {"LOCALAPPDATA": str(root)}, stdout=subprocess.PIPE,
        stdin=subprocess.DEVNULL, creationflags=subprocess.CREATE_NEW_CONSOLE,
        startupinfo=startup,
    )
    watchdog = Timer(40, child.kill)
    watchdog.start()
    try:
        assert child.stdout.readline().startswith(b"HEARTBEAT rehearsal ")
        yield child
    finally:
        watchdog.cancel()
        if child.poll() is None:
            child.kill()
        child.wait(timeout=5)
        child.stdout.close()


def connect(name: str, pid: int):
    until = time.monotonic() + 5
    while True:
        try:
            pipe = open(name, "r+b", buffering=0)
            break
        except OSError as exc:
            if exc.winerror != 231 or time.monotonic() >= until:
                raise
            time.sleep(0.01)
    server = wintypes.DWORD()
    handle = wintypes.HANDLE(msvcrt.get_osfhandle(pipe.fileno()))
    if not KERNEL.GetNamedPipeServerProcessId(handle, ctypes.byref(server)) or server.value != pid:
        pipe.close()
        raise RuntimeError("pipe does not belong to this test host")
    return pipe


def read_exact(pipe, size: int) -> bytes:
    data = bytearray()
    while len(data) < size:
        chunk = pipe.read(size - len(data))
        if not chunk:
            raise EOFError("terminal disconnected")
        data.extend(chunk)
    return bytes(data)


def send(pipe, name: str, request: dict) -> None:
    data = codec.encode_protobuf(name, request, LIMITS)
    frame = struct.pack("<I", len(data)) + data
    assert pipe.write(frame) == len(frame)


def exchange(pipe, name: str, request: dict, response: str) -> dict:
    send(pipe, name, request)
    size, = struct.unpack("<I", read_exact(pipe, 4))
    assert 0 < size <= LIMITS.max_bytes
    return codec.decode_protobuf(response, read_exact(pipe, size), LIMITS)


def authenticate(pipe, token: bytes) -> dict:
    request = dict(schema_version=1, request_id="auth", installation_id="desktop",
                   token=token, protocol_versions=[1])
    reply = exchange(pipe, "TerminalAuthenticate", request, "TerminalAuthResult")
    ingress.auth_pair(request, reply, LIMITS)
    return reply


def request_for(session, kind, **payload):
    return dict(schema_version=1, request_id=next(iter(payload)), protocol_version=1,
                session_id=session["session_id"], kind=int(kind), **payload)


def dispatch(pipe, session, kind, **payload):
    request = request_for(session, kind, **payload)
    reply = exchange(pipe, "TerminalRequest", request, "TerminalResponse")
    ingress.response(request, reply, session, LIMITS)
    return reply


def upload(pipe, session, name, value):
    data = codec.encode_canonical(name, value, LIMITS)
    ref = dict(schema_version=1, object_type=codec.SCHEMA["messages"][name]["tag"],
               resource_version="1", byte_length=len(data),
               digest=bytes.fromhex(codec.digest(name, value, LIMITS)))
    chunk = dict(schema_version=1, upload_id=ref["digest"].hex(), object=ref, offset=0, data=data, final_chunk=True)
    reply = dispatch(pipe, session, Kind.INGRESS_UPLOAD, upload=chunk)
    assert reply["upload"]["status"] == UploadStatus.UPLOAD_COMPLETE
    return ref


def observation(pipe, session, fixtures, index):
    payload = upload(pipe, session, "UserText", dict(schema_version=1, text=f"synthetic host text {index}"))
    source = fixtures["SourceMetadata"] | dict(source_span=payload, epistemic_type=int(EpistemicType.USER_ASSERTION))
    provenance = upload(pipe, session, "SourceMetadata", source)
    return fixtures["TerminalEvent"] | dict(kind=int(EventKind.OBSERVATION), id=f"host-{index}",
        stable_source_key=f"01ARZ3NDEKTSV4RRFFQ{index:07d}", payload_ref=payload,
        payload_digest=payload["digest"], provenance=provenance)


def durable(pipe, session, event):
    receipt = dispatch(pipe, session, Kind.INGRESS_EVENT, event=event)["receipt"]
    assert receipt["stage"] == ReceiptStage.RECEIVED_DURABLE
    assert receipt["stable_source_key"] == event["stable_source_key"]
    assert receipt["event_id"] == event["id"]
    return receipt


def raw_events(root):
    uri = (root / "Nemu/core/rehearsal/store.sqlite").as_uri() + "?mode=ro"
    with closing(sqlite3.connect(uri, uri=True)) as db:
        return [codec.decode_canonical("EventEnvelope", row[0], LIMITS)
                for row in db.execute("SELECT event FROM raw_ingress ORDER BY position")]


def session_for(pipe, token):
    auth = authenticate(pipe, token)
    assert auth["status"] == AuthStatus.AUTHENTICATED
    session = auth["session"]
    assert set(session["capabilities"]) == {
        Kind.INGRESS_EVENT, Kind.INGRESS_QUERY, Kind.INGRESS_APPROVAL, Kind.INGRESS_CANCEL, Kind.INGRESS_UPLOAD,
    }
    return session


def test_host(binary: str, root: Path):
    refused = subprocess.run([binary, "--lineage", "life"], env=os.environ | {"LOCALAPPDATA": str(root)},
                             capture_output=True, timeout=5)
    assert refused.returncode and not (root / "Nemu").exists()
    sid = powershell("[Security.Principal.WindowsIdentity]::GetCurrent().User.Value", root).decode().strip()
    pipe_name = rf"\\.\pipe\nemu-core-{sid}"
    token_path = root / "Nemu/terminal.token"
    samples = json.loads((ROOT / "core/contracts/golden/legal.json").read_text(encoding="utf-8"))
    fixtures = {v["request"]["name"]: from_json(v["request"]["name"], v["request"]["value"])
                for v in samples if v["request"]["name"] in {"TerminalEvent", "SourceMetadata", "GrowthControl"}}
    sessions = []
    accepted = []
    with running(binary, root) as child:
        protected = token_path.read_bytes()
        # Private subprocess pipe only. No token in argv, environment, logs or evidence.
        token = powershell(
            "Add-Type -AssemblyName System.Security\n"
            "$b=[Security.Cryptography.ProtectedData]::Unprotect("
            "[IO.File]::ReadAllBytes((Join-Path $PWD 'Nemu/terminal.token')),$null,'CurrentUser')\n"
            "[Console]::OpenStandardOutput().Write($b,0,$b.Length)", root)
        assert len(token) == 32
        with connect(pipe_name, child.pid) as pipe:
            assert authenticate(pipe, bytes(32))["reason"] == Reason.AUTH_FAILED
        with connect(pipe_name, child.pid) as pipe:
            pipe.write(struct.pack("<I", LIMITS.max_bytes + 1))
            try:
                assert not pipe.read(1)
            except BrokenPipeError:
                pass
        with connect(pipe_name, child.pid) as pipe:
            session = session_for(pipe, token)
            sessions.append(session["session_id"])
            growth = dispatch(pipe, session, Kind.INGRESS_GROWTH, growth=fixtures["GrowthControl"])
            assert growth["reason"] == Reason.CAPABILITY_DENIED
            first = observation(pipe, session, fixtures, 0)
            rejected = dispatch(pipe, session, Kind.INGRESS_EVENT, event=first | {"parents": ["unverified"]})
            assert rejected["reason"] == Reason.CAPABILITY_DENIED
            for index in range(20):
                event = first if index == 0 else observation(pipe, session, fixtures, index)
                receipt = durable(pipe, session, event)
                assert receipt["raw_log_position"] == index
                accepted.append((event, receipt))
            # Kill immediately after consuming the last durable receipt.
            child.kill()
            child.wait(timeout=5)

    with running(binary, root) as child:
        recovered = raw_events(root)
        assert len(recovered) == len(accepted) == 20
        for stored, (event, receipt) in zip(recovered, accepted, strict=True):
            assert all(stored[k] == v for k, v in event.items())
            assert stored["ingest_time_ns"] == receipt["ingest_time_ns"]
            assert stored["raw_log_position"] == receipt["raw_log_position"]
            assert stored["session"] == sessions[0] and stored["device"] == "desktop"
        with connect(pipe_name, child.pid) as pipe:
            session = session_for(pipe, token)
            sessions.append(session["session_id"])
            unacknowledged = observation(pipe, session, fixtures, 20)
            send(pipe, "TerminalRequest", request_for(session, Kind.INGRESS_EVENT, event=unacknowledged))
            # No response read. Either storage outcome is legal at this boundary.
            child.kill()
            child.wait(timeout=5)

    with running(binary, root) as child:
        before_retry = raw_events(root)
        assert before_retry[:20] == recovered and len(before_retry) in (20, 21)
        if len(before_retry) == 21:
            assert all(before_retry[20][k] == v for k, v in unacknowledged.items())
        with connect(pipe_name, child.pid) as pipe:
            session = session_for(pipe, token)
            sessions.append(session["session_id"])
            retry = observation(pipe, session, fixtures, 20)
            assert retry == unacknowledged
            receipt = durable(pipe, session, retry)
            assert receipt["raw_log_position"] == len(before_retry)
        start = time.monotonic()
        subprocess.run([sys.executable, "-I", "-B", __file__, "--stop", str(child.pid)],
                       creationflags=subprocess.CREATE_NO_WINDOW, check=True, timeout=STOP_SECONDS)
        assert child.wait(timeout=max(0, STOP_SECONDS - (time.monotonic() - start))) == 0
        print(f"console stop: {time.monotonic() - start:.3f} s")

    with running(binary, root) as child:
        assert token_path.read_bytes() == protected
        final = raw_events(root)
        assert final[:-1] == before_retry and all(final[-1][k] == v for k, v in retry.items())
        with connect(pipe_name, child.pid) as pipe:
            sessions.append(session_for(pipe, token)["session_id"])
        # Every accepted handshake already has a durable record, even without an event.
        with closing(sqlite3.connect((root / "Nemu/rehearsal-auth.sqlite").as_uri() + "?mode=ro", uri=True)) as db:
            rows = db.execute("SELECT record_id,principal_id,installation_id,session_id FROM local_auth_v1").fetchall()
        assert len(rows) == len(set(sessions)) == 4
        assert {r[3] for r in rows} == set(sessions)
        assert all(r[0] == "auth-v1-" + r[3] and r[2] == "desktop" for r in rows)
        assert {r[1] for r in rows} == {hashlib.sha256(b"nemu/windows-principal/v1\0" + sid.encode()).hexdigest()}
    with closing(sqlite3.connect((root / "Nemu/core/rehearsal/store.sqlite").as_uri() + "?mode=ro", uri=True)) as db:
        reasons = [codec.decode_canonical("IngressRejection", row[0], LIMITS)["reason"]
                   for row in db.execute("SELECT record FROM ingress_rejections ORDER BY position")]
    assert sorted(reasons) == sorted([Reason.AUTH_FAILED, Reason.INVALID_FRAME, Reason.CAPABILITY_DENIED, Reason.CAPABILITY_DENIED])
    print(f"20 durable receipts recovered, unacknowledged retry position {receipt['raw_log_position']}, "
          f"{len(final)} D records, 4 authenticated sessions, 4 audited denials")


if __name__ == "__main__":
    if sys.argv[1] == "--stop":
        stop_private_console(int(sys.argv[2]))
    else:
        with tempfile.TemporaryDirectory(prefix="nemu-host-test-") as directory:
            test_host(sys.argv[1], Path(directory))
