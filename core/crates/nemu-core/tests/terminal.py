"""Exercise the real rehearsal process with an isolated LOCALAPPDATA and console."""
from __future__ import annotations

import ctypes
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
from contextlib import contextmanager

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "workers/irl/generated"))
from echo import ingress, runtime as codec
from echo.bridge import from_json
from echo.types import AuthStatus, EpistemicType, IngressKind as Kind, IngressReason as Reason, UploadStatus

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


def exchange(pipe, name: str, request: dict, response: str) -> dict:
    data = codec.encode_protobuf(name, request, LIMITS)
    frame = struct.pack("<I", len(data)) + data
    assert pipe.write(frame) == len(frame)
    size, = struct.unpack("<I", read_exact(pipe, 4))
    assert 0 < size <= LIMITS.max_bytes
    return codec.decode_protobuf(response, read_exact(pipe, size), LIMITS)


def authenticate(pipe, token: bytes) -> dict:
    request = dict(schema_version=1, request_id="auth", installation_id="desktop",
                   token=token, protocol_versions=[1])
    reply = exchange(pipe, "TerminalAuthenticate", request, "TerminalAuthResult")
    ingress.auth_pair(request, reply, LIMITS)
    return reply


def dispatch(pipe, session, kind, **payload):
    request = dict(schema_version=1, request_id=next(iter(payload)), protocol_version=1,
                   session_id=session["session_id"], kind=int(kind), **payload)
    reply = exchange(pipe, "TerminalRequest", request, "TerminalResponse")
    ingress.response(request, reply, session, LIMITS)
    return reply


def upload(pipe, session, name, value):
    data = codec.encode_canonical(name, value, LIMITS)
    ref = dict(schema_version=1, object_type=codec.SCHEMA["messages"][name]["tag"],
               resource_version="1", byte_length=len(data),
               digest=bytes.fromhex(codec.digest(name, value, LIMITS)))
    chunk = dict(schema_version=1, upload_id=name, object=ref, offset=0, data=data, final_chunk=True)
    reply = dispatch(pipe, session, Kind.INGRESS_UPLOAD, upload=chunk)
    assert reply["upload"]["status"] == UploadStatus.UPLOAD_COMPLETE
    return ref


def check_terminal(pipe, token, fixtures):
    auth = authenticate(pipe, token)
    assert auth["status"] == AuthStatus.AUTHENTICATED
    session = auth["session"]
    assert set(session["capabilities"]) == {
        Kind.INGRESS_EVENT, Kind.INGRESS_QUERY, Kind.INGRESS_APPROVAL, Kind.INGRESS_CANCEL, Kind.INGRESS_UPLOAD,
    }
    growth = dispatch(pipe, session, Kind.INGRESS_GROWTH, growth=fixtures["GrowthControl"])
    assert growth["reason"] == Reason.CAPABILITY_DENIED
    payload = upload(pipe, session, "UserText", dict(schema_version=1, text="synthetic host text"))
    source = fixtures["SourceMetadata"] | dict(source_span=payload, epistemic_type=int(EpistemicType.USER_ASSERTION))
    provenance = upload(pipe, session, "SourceMetadata", source)
    event = fixtures["TerminalEvent"] | dict(stable_source_key="01ARZ3NDEKTSV4RRFFQ69G5FAV",
        payload_ref=payload, payload_digest=payload["digest"], provenance=provenance)
    # The card explicitly requires denial until a source-authority predicate exists.
    for index in range(20):
        observation = event | dict(id=f"host-{index}", stable_source_key=f"01ARZ3NDEKTSV4RRFFQ{index:07d}")
        reply = dispatch(pipe, session, Kind.INGRESS_EVENT, event=observation)
        assert reply["reason"] == Reason.CAPABILITY_DENIED and "receipt" not in reply
    print("user_text: 20 CAPABILITY_DENIED, 0 RECEIVED_DURABLE")


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
    with running(binary, root) as child:
        protected = token_path.read_bytes()
        # The private subprocess stdout pipe carries the temporary credential
        # directly into memory. Neither child output nor token is logged.
        token = powershell(
            "Add-Type -AssemblyName System.Security\n"
            "$b=[Security.Cryptography.ProtectedData]::Unprotect("
            "[IO.File]::ReadAllBytes((Join-Path $PWD 'Nemu/terminal.token')),$null,'CurrentUser')\n"
            "[Console]::OpenStandardOutput().Write($b,0,$b.Length)", root)
        assert len(token) == 32
        with connect(pipe_name, child.pid) as pipe:
            assert authenticate(pipe, bytes(32))["reason"] == Reason.AUTH_FAILED
        with connect(pipe_name, child.pid) as pipe:
            check_terminal(pipe, token, fixtures)
        with connect(pipe_name, child.pid) as pipe:
            pipe.write(struct.pack("<I", LIMITS.max_bytes + 1))
            try:
                assert not pipe.read(1)
            except BrokenPipeError:
                pass
        start = time.monotonic()
        subprocess.run([sys.executable, "-I", "-B", __file__, "--stop", str(child.pid)],
                       creationflags=subprocess.CREATE_NO_WINDOW, check=True, timeout=STOP_SECONDS)
        assert child.wait(timeout=max(0, STOP_SECONDS - (time.monotonic() - start))) == 0
        print(f"console stop: {time.monotonic() - start:.3f} s")
    db = root / "Nemu/core/rehearsal/store.sqlite"
    with sqlite3.connect(db.as_uri() + "?mode=ro", uri=True) as conn:
        assert conn.execute("SELECT count(*) FROM raw_ingress").fetchone() == (0,)
        reasons = [codec.decode_canonical("IngressRejection", row[0], LIMITS)["reason"]
                   for row in conn.execute("SELECT record FROM ingress_rejections ORDER BY position")]
        assert reasons == [Reason.AUTH_FAILED] + [Reason.CAPABILITY_DENIED] * 21 + [Reason.INVALID_FRAME]
    conn.close()
    with running(binary, root) as child:
        assert token_path.read_bytes() == protected
        with connect(pipe_name, child.pid) as pipe:
            assert authenticate(pipe, token)["status"] == AuthStatus.AUTHENTICATED


if __name__ == "__main__":
    if sys.argv[1] == "--stop":
        stop_private_console(int(sys.argv[2]))
    else:
        with tempfile.TemporaryDirectory(prefix="nemu-host-test-") as directory:
            test_host(sys.argv[1], Path(directory))
