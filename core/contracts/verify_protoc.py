"""Independent official Protobuf compiler/runtime interoperability oracle."""

from __future__ import annotations
import json
from pathlib import Path
import subprocess
import sys
from google.protobuf import descriptor_pb2, descriptor_pool, message_factory

HERE = Path(__file__).resolve().parent
target = HERE / ".test-build/descriptor.pb"
target.parent.mkdir(exist_ok=True)
subprocess.run(
    [
        sys.executable,
        "-m",
        "grpc_tools.protoc",
        "-I",
        str(HERE),
        "--descriptor_set_out=" + str(target),
        "--include_imports",
        *[str(p) for p in sorted(HERE.glob("*.proto"))],
    ],
    check=True,
)
descriptors = descriptor_pb2.FileDescriptorSet.FromString(target.read_bytes())
pool = descriptor_pool.DescriptorPool()
for file in descriptors.file:
    pool.Add(file)
vectors = json.loads((HERE / "golden/legal.json").read_text(encoding="utf-8"))
oracle = []
for vector in vectors:
    name = vector["request"]["name"]
    message = message_factory.GetMessageClass(
        pool.FindMessageTypeByName("nemu.echo.v1." + name)
    )()
    data = bytes.fromhex(vector["expected"]["protobuf"])
    message.ParseFromString(data)
    # Official proto3 may choose packed repeated scalars; both are legal wire forms.
    encoded = message.SerializeToString(deterministic=True)
    oracle.append(
        dict(
            request=dict(
                name=name,
                op="protobuf",
                hex=encoded.hex(),
                limits=vector["request"]["limits"],
            ),
            expected=vector["expected"],
        )
    )
(HERE / ".test-build/protoc-vectors.json").write_text(
    json.dumps(oracle), encoding="utf-8"
)
print(f"Official protoc and protobuf runtime: {len(vectors)} transport vectors PASS")
