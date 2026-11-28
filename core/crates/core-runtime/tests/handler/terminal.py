"""Live P5-53 gateway, throwaway DPAPI token and PID-checked SID pipe."""
import os
from pathlib import Path
import struct
import sys
from threading import Timer

# Reuse the HOST-01 wire client, PID check, generated codec and upload helpers.
sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "nemu-core/tests"))
import terminal as host


def evidence(name, body):
    target = os.getenv("NEMU_P5_53_EVIDENCE_DIR")
    if target:
        with (Path(target) / name).open("xb") as stream:
            stream.write(body)


def read_frame(pipe):
    size, = struct.unpack("<I", host.read_exact(pipe, 4))
    assert 0 < size <= host.LIMITS.max_bytes
    return host.read_exact(pipe, size)


def settlement(pipe, session, event, stage):
    request = host.request_for(session, host.Kind.INGRESS_EVENT, event=event)
    data = read_frame(pipe)
    evidence(f"restart-{event['id']}-settlement.pb", data)
    reply = host.codec.decode_protobuf("TerminalResponse", data, host.LIMITS)
    host.ingress.response(request, reply, session, host.LIMITS)
    assert reply["receipt"]["stage"] == stage
    assert reply["receipt"]["event_id"] == event["id"]


def run(directory, endpoint, pid, phase):
    token = host.powershell(
        "Add-Type -AssemblyName System.Security\n"
        "$b=[Security.Cryptography.ProtectedData]::Unprotect("
        "[IO.File]::ReadAllBytes((Join-Path $PWD 'token.dpapi')),$null,'CurrentUser')\n"
        "[Console]::OpenStandardOutput().Write($b,0,$b.Length)", directory)
    with host.connect(endpoint, pid) as pipe:
        session = host.authenticate(pipe, token)["session"]
        # No TerminalRequest has been sent. This must be an active state push.
        data = read_frame(pipe)
        host.codec.decode_protobuf("RunStateV2", data, host.LIMITS)
        evidence(f"{phase}-authenticated-state.pb", data)
        (directory / "state.pb").write_bytes(data)
        event = host.observation(pipe, session, 0)
        rejected = host.dispatch(pipe, session, host.Kind.INGRESS_EVENT,
                                 event=event | {"parents": ["unknown-parent"]})
        assert rejected["reason"] == host.Reason.CAPABILITY_DENIED
        if phase == "failure":
            # A process failure can interrupt even the durable reply in transit.
            # Every complete frame after admission must be that one receipt.
            request = host.request_for(session, host.Kind.INGRESS_EVENT, event=event)
            host.send(pipe, "TerminalRequest", request)
            received = 0
            try:
                while True:
                    body = read_frame(pipe)
                    evidence("failure-reply.pb", body)
                    reply = host.codec.decode_protobuf("TerminalResponse", body, host.LIMITS)
                    host.ingress.response(request, reply, session, host.LIMITS)
                    assert reply["receipt"]["stage"] == host.ReceiptStage.RECEIVED_DURABLE
                    received += 1
                    assert received == 1
            except (EOFError, BrokenPipeError):
                return
        durable = host.durable(pipe, session, event)
        evidence("restart-host-0-durable-receipt.pb", host.codec.encode_protobuf("ReceiptV2", durable, host.LIMITS))
        settlement(pipe, session, event, host.ReceiptStage.FILTERED)
        new = host.observation(pipe, session, 1)
        durable = host.durable(pipe, session, new)
        evidence("restart-host-1-durable-receipt.pb", host.codec.encode_protobuf("ReceiptV2", durable, host.LIMITS))
        settlement(pipe, session, new, host.ReceiptStage.CORE_COMMITTED)
        cancel = host.dispatch(pipe, session, host.Kind.INGRESS_CANCEL,
                               cancel={"schema_version": 1, "output_id": "missing"})
        assert cancel["reason"] == host.Reason.CAPABILITY_DENIED
    # Authenticate a new connection after the settlement to sample the new head.
    with host.connect(endpoint, pid) as pipe:
        host.authenticate(pipe, token)
        data = read_frame(pipe)
        evidence("restart-final-head-state.pb", data)
        (directory / "state.pb").write_bytes(data)


if __name__ == "__main__":
    watchdog = Timer(45, lambda: os._exit(2))
    watchdog.start()
    try:
        run(Path(sys.argv[1]), sys.argv[2], int(sys.argv[3]), sys.argv[4])
    finally:
        watchdog.cancel()
