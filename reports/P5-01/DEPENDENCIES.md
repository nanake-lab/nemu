# P5-01 dependency inventory

Rust runtime and transitive dependencies, verified from cargo metadata --locked. All are permissively licensed. No Node runtime dependency was added; zod 4.6.7 already belongs to @nemu/contracts. Python codec runtime uses the standard library only.

| Name | Version | License |
| --- | --- | --- |
| block-buffer | 0.12.1 | MIT OR Apache-2.0 |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 |
| const-oid | 0.10.2 | Apache-2.0 OR MIT |
| cpufeatures | 0.3.1 | MIT OR Apache-2.0 |
| crypto-common | 0.2.2 | MIT OR Apache-2.0 |
| digest | 0.11.3 | MIT OR Apache-2.0 |
| hybrid-array | 0.4.15 | MIT OR Apache-2.0 |
| itoa | 1.0.18 | MIT OR Apache-2.0 |
| libc | 0.2.190 | MIT OR Apache-2.0 |
| memchr | 2.8.3 | Unlicense OR MIT |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| serde | 1.0.229 | MIT OR Apache-2.0 |
| serde_core | 1.0.229 | MIT OR Apache-2.0 |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 |
| serde_json | 1.0.154 | MIT OR Apache-2.0 |
| sha2 | 0.11.1 | MIT OR Apache-2.0 |
| syn | 3.0.9 | MIT OR Apache-2.0 |
| typenum | 1.20.1 | MIT OR Apache-2.0 |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| zmij | 1.0.23 | MIT |

Development-only conformance dependencies: grpcio-tools 1.84.0 and grpcio 1.84.0 (Apache-2.0), protobuf 7.36.2 (BSD-3-Clause), mypy 1.20.2 (MIT), ruff 0.15.6 (MIT), pytest 9.1.1 (MIT). Exact transitive tooling versions are locked in core/contracts/uv.lock. Existing TypeScript/Node tools are reused.

Files over 400 lines: generated schema/DTO files and golden JSON are deliberately machine-produced from one IDL source and retained for review. The formatted TypeScript runtime template is one cohesive codec module (461 nonblank lines before final amendments), kept together to audit canonical/protobuf validation symmetry; no application logic or I/O is mixed into it.
