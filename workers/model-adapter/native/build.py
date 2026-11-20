"""Build the Windows shim from an x64 MSVC developer prompt, against pinned DLLs."""
from __future__ import annotations

import argparse
from pathlib import Path
import re
import subprocess


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--engine", type=Path, required=True)
    parser.add_argument("--source", type=Path, required=True, help="b11669 source tree")
    parser.add_argument("--headers", type=Path, help="optional additional b11669 header root")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--measurement-control", choices=["plain", "drop-vector"])
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(exist_ok=True, parents=True)
    libraries = []
    for library in ("llama", "llama-common", "ggml", "ggml-base", "mtmd"):
        dll = args.engine / (library + ".dll")
        listing = subprocess.check_output(["dumpbin", "/nologo", "/exports", str(dll)], text=True)
        exports = re.findall(r"^\s*\d+\s+[\dA-F]+\s+[\dA-F]+\s+(\S+)", listing, re.M)
        if not exports:
            raise RuntimeError(f"No exports in {library}")
        definition = out / (library + ".def")
        definition.write_text(f'LIBRARY "{dll.name}"\nEXPORTS\n' + "\n".join(exports), "ascii")
        lib = out / (library + ".lib")
        subprocess.run(["lib", "/nologo", "/machine:x64", f"/def:{definition}", f"/out:{lib}"],
                       cwd=out, check=True, stdout=subprocess.DEVNULL)
        libraries.append(str(lib))
    source = args.source.resolve()
    extra = (args.headers or source).resolve()
    control = {"plain": ["/DP513_PLAIN_REFERENCE"], "drop-vector": ["/DP513_DROP_VECTOR"]}
    dll_name = "carrier" if not args.measurement_control else "measurement-" + args.measurement_control
    subprocess.run(["cl", "/nologo", "/std:c++17", "/utf-8", "/EHsc", "/MD", "/O2", "/W4", "/WX",
                    *control.get(args.measurement_control, []),
                    "/fp:precise", "/LD", f"/I{source / 'include'}", f"/I{source / 'ggml/include'}",
                    f"/I{extra / 'common'}", f"/I{extra / 'tools/mtmd'}",
                    str(Path(__file__).with_name("carrier.cpp").resolve()), *libraries,
                    "/link", "/Brepro", f"/OUT:{dll_name}.dll"], cwd=out, check=True)


if __name__ == "__main__":
    main()
