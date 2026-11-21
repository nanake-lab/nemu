"""Link W19 and runtime as a test consumer without changing the workspace lock."""

import json
import os
from pathlib import Path
import subprocess
import tempfile


def main() -> None:
    core = Path(__file__).resolve().parents[5]
    package = core / "crates" / "core-runtime"
    # Cargo reports the actual feature-selected rlibs and native link paths,
    # including paths from a shared target directory or a custom Cargo config.
    built = subprocess.run(
        ["cargo", "test", "--workspace", "--locked", "--no-run", "--message-format=json"],
        cwd=core, stdout=subprocess.PIPE, check=True,
    )
    libraries: dict[str, Path] = {}
    native: set[str] = set()
    for line in built.stdout.splitlines():
        record = json.loads(line)
        if record["reason"] == "build-script-executed":
            native.update(record["linked_paths"])
        if record["reason"] == "compiler-artifact" and "lib" in record["target"]["kind"]:
            for filename in record["filenames"]:
                if filename.endswith(".rlib"):
                    libraries[record["target"]["name"].replace("-", "_")] = Path(filename)
    args = [
        "rustc", "--test", "--edition=2024", str(package / "src/cognition/tests/w19.rs"),
        "-L", f"dependency={libraries['core_runtime'].parent}",
    ]
    for name in ("core_types", "core_runtime", "core_store", "core_policy", "core_kernel", "fact_view"):
        args.extend(["--extern", f"{name}={libraries[name]}"])
    for directory in sorted(native):
        args.extend(["-L", directory])
    with tempfile.TemporaryDirectory() as temp:
        executable = Path(temp) / ("w19-test.exe" if os.name == "nt" else "w19-test")
        env = dict(os.environ, CARGO_MANIFEST_DIR=str(package))
        subprocess.run([*args, "-o", str(executable)], env=env, cwd=core, check=True)
        subprocess.run(
            [str(executable), "--exact", "w19_historical_reaction_after_internal_publication", "--nocapture"],
            env=env, cwd=core, check=True,
        )


if __name__ == "__main__":
    main()
