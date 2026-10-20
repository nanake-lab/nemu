# P0-06 v1.0 scope and acceptance routing

Authority: Echo Core README §3; System Plan (S) §§4,26,28–30; Operators (O) §20; Frozen Architecture (F) §§16–17; Delivery (D) Part I §§10–11; `tasks/P5-echo-core.md`. Original-file aliases resolve through [baseline.md](baseline.md). This is a routing plan, **not passed acceptance evidence**. All core acceptance below is `not_run` at W00. P0-02 contract tests are not substitutes.

## Work packages

| Package | Task | v1.0 boundary |
| --- | --- | --- |
| W00 | P0-06 | Baseline, pending conflict approval |
| W01 | P5-01 | IDL/canonical encoding; inactive growth objects are types only |
| W02 | P5-02 | Single writer and object storage |
| W03 | P5-08 | Full stage order, clocks and rehearsal degradation |
| W04 | P5-09 | Authorized effect projection |
| W05 | P5-10, P5-11; integration P5-19 | Fast state and drive proposals |
| W06 | v1.x, task not assigned | χ/K discovery excluded |
| W07 | v1.x, task not assigned | Trials/qualification excluded |
| W08 | v1.x, task not assigned | G activation/lifecycle excluded |
| W09 | v1.x, task not assigned | C candidates excluded; W00-C01 and missing upstream |
| W10 | v1.x, task not assigned | C handoff/growth excluded; W00-C01 and missing upstream |
| W11 | P5-12 | J/readout compiler |
| W12 | P0-07 feasibility; P5-13 implementation | Real model carrier |
| W13 | P5-14, P5-20 | Lifecycle and autonomous scheduling |
| W14 | P5-15 | Structural replay, local backup, genesis/birth control |
| W15 | P5-16 | Basic observability only |
| W16 | P5-18 | M2 subset; not M7 behavior certification |
| W17 | P5-17 | Genesis calibration subset; user participates |
| W18 | P5-03 | Ingress |
| W19 | P5-04 | Bounded fact view |
| W20 | P5-06, P5-07 | Semantic governance and IRL |
| W21 | P6-01 | Task protocol/output |
| W22 | P5-05 | Identity, permission, worker isolation |
| W23 | v1.x, task not assigned | Multi-terminal authority, remote encrypted backup/failover |

G/C remain in event order as audited identity stages; R empty, θ genesis only. Nemu N0–N4 milestones are distinct from Echo Core M0–M7. P5-01 requires merged P0-06 and P0-02; this document does not waive dependencies.

## Operator acceptance (all 45 IDs retained)

| IDs | Responsible task / disposition |
| --- | --- |
| F01, F02, F03, F05 | P5-10; F03 structural only at M2, deployment calibration P5-17; aggregate P5-18 |
| F04 | v1.x M7; information-only calibration P5-17, no functional pass |
| F06 | P5-09 structural routing; behavioral part v1.x; aggregate P5-18 |
| F07 | P5-11/P5-19 integration; aggregate P5-18 |
| G01, G02, G03, G04, G05, G06, G07, G08, G09, G10, G11 | v1.x W06–W08; no nonzero-G success claimed |
| C01, C02, C03, C04, C05, C06, C07, C08, C09, C10, C11, C12 | v1.x W09–W10; W00-C01 must be resolved; no nonzero-C success claimed |
| J01, J05 | P0-07 probe, P5-13 implementation; aggregate P5-18 |
| J02, J03, J04, J07 | v1.x M7; P5-17 information-only observations, no certification |
| J06 | P5-09/P5-12 structural fixture; aggregate P5-18 |
| X01 | P5-18 integration, capabilities from P5-01/P5-05 |
| X02 | P5-08; aggregate P5-18 |
| X03 | P5-02/P5-08 crash injection on v1.0 stages; aggregate P5-18 |
| X04 | P5-14; aggregate P5-18 |
| X05 | P5-15; aggregate P5-18 |
| X06 | P5-08 task-level acceptance; not added to README M2 gate subset |
| X07, X08 | v1.x full-growth acceptance; preserve IDs, not_run |

## System acceptance (all 25 IDs retained)

“M2” means explicitly listed in README §3 for P5-18. Additional card-level checks are not silently removed simply because they are outside that gate subset.

| ID | Tasks | Disposition |
| --- | --- | --- |
| SYS01 | P5-03/P5-02 | M2 |
| SYS02 | P5-06/P5-07/P5-19 | M2 |
| SYS03 | P5-06/P5-07/P5-17 | Card structural/accuracy report, no M7 claim |
| SYS04 | P5-06/P5-09 | Card acceptance |
| SYS05 | P5-04 | M2 |
| SYS06 | P6-01/P5-08 | M2 |
| SYS07 | P6-01/P5-03 | W21/W18 integration check, outside explicit M2 gate |
| SYS08 | P5-14/P5-20/P6-01 | M2 |
| SYS09 | P5-05/P5-14 | M2 |
| SYS10 | P5-20/P5-16 | Card acceptance |
| SYS11 | P5-14 | M2 |
| SYS12 | P5-14/P5-03 | M2 |
| SYS13 | P6-01/P5-14/P5-15 | M2 |
| SYS14 | P5-08/P5-19; P5-07/P6-01 | M2 |
| SYS15 | P5-15 | M2 |
| SYS16 | P5-15 | Card acceptance |
| SYS17 | P5-13/P5-15 | Required whenever migration is implemented; failed migration retains old system |
| SYS18 | v1.x W18/W23 | Multi-terminal path excluded |
| SYS19 | P5-15 local portion; v1.x W23 | Local card acceptance; remote portion not_run |
| SYS20 | v1.x W23/W22 | Authority failover excluded |
| SYS21 | P5-01/P5-18 | M2 |
| SYS22 | P5-02/P5-16/P5-18 | M2 |
| SYS23 | P5-08/P5-20 | M2 clock portion; R/θ growth expiry tests v1.x |
| SYS24 | v1.x W16/W17 | Full nonzero G/C and M7 functional chain excluded |
| SYS25 | P5-01/P5-06/P5-15/P5-18 | M2; keep despite S §32 count mismatch W00-C02 |

## Invariants and enforcement routing

All 22 retain structural checks. CI/offline M2 set: I01–I05, I11, I14–I16, I18, I21. “v1.x” below refers to positive growth behavior, not permission to violate the invariant now. P5-18 aggregates checks; no checks are implemented by this documentation card.

| ID | Tasks / check mechanism |
| --- | --- |
| I01 | P5-02/P5-15 append-only commit-chain/offline verifier |
| I02 | P5-02/P5-05 staged capability and OS isolation |
| I03 | P5-01/P5-08 typed write capabilities |
| I04 | P5-03/P5-08/P5-10 event/anchor atomicity tests |
| I05 | P5-10/P5-17 finite bounds/contraction tests |
| I06 | P5-09/P5-20/P6-01 authorized-view isolation; growth positive tests v1.x W08 |
| I07 | P5-09 unique owner projection; handoff v1.x W10 |
| I08 | P5-01 structural identity separation; revision/handoff v1.x W08/W10 |
| I09 | P5-04 completeness; qualification v1.x W06/W07 |
| I10 | Structural no-growth P5-08; meta lineage v1.x W08 |
| I11 | P5-10/P5-12/P5-20 pure time/read projection tests |
| I12 | Structural no-growth P5-08; invalidation barriers v1.x W08/W10 |
| I13 | P5-01/P5-05/P5-08 no θ writer except C; actual growth v1.x W10 |
| I14 | P5-01/P5-02/P5-08 exact binding/stale proposal rejection |
| I15 | P5-12/P5-13/P5-17 actual hook receipts; E8 explicitly non-strict |
| I16 | P5-14/P5-15 replay does not execute or mint evidence |
| I17 | P5-16/P5-17 scoped, versioned measurements; full certificates v1.x |
| I18 | P5-01/P5-06/P5-18 state-completeness perturbation tests (SYS25) |
| I19 | P5-08 no-growth structural guard; candidate-free witness v1.x W06/W07 |
| I20 | P5-08 no-growth guard; v1.x W07/W09/W10; W00-C01 pending |
| I21 | P5-08/P5-09 pure projections; no write-on-read |
| I22 | P5-09/P5-20 structural validity gating; full θ expiry v1.x W09/W10 |

## Earlier specification acceptance

| IDs / original | Tasks / boundary |
| --- | --- |
| F §16.1, §16.13 | P5-02/P5-08, M2 |
| F §16.2, §16.3, §16.9, §16.12 | v1.x growth; single-root conflict W00-C01 retained |
| F §16.4, §16.5 | P5-08/P5-14/P5-20, M2 |
| F §16.6 | P6-01/P5-18, M2 protocol-isolation portion; growth portion v1.x |
| F §16.7 | P5-13/P5-18 carrier failure portion M2; growth behavioral review v1.x |
| F §16.8 | P5-12/P5-13/P5-18 committed snapshot/version binding, M2 |
| F §16.10 | P5-15 card acceptance |
| F §16.11 | P5-15, M2 |
| F §16.14 | P5-14/P5-15/P5-20, M2 |
| F §16.15, §16.16 | P5-06/P5-07, M2 |
| F §16.17 | P5-10/P5-17, M2 structural portion |
| F §16.18, §16.19, §16.20 | v1.x W06–W10 |
| F §17.1–§17.10 | v1.x M7; P5-17 observations are informational |
| D Part I §10.1, §10.2 | P5-02/P5-08; M2 |
| D Part I §10.3 | P5-06/P5-07/P5-19; M2 |
| D Part I §10.4 | P5-08/P5-19; M2 |
| D Part I §10.5, §10.6 | P5-10/P5-11/P5-19; M2 |
| D Part I §10.7 | P5-15; M2 |
| D Part I §10.8 | P5-01; M2 |
| D Part I §10.9 | P5-05; M2 |
| D Part I §11 first integration milestone | P5-19; M2 |
| D Part II §8 / R §8 growth matrix | v1.x W06–W10, not substituted by identity stages |

S §30 attack-review examples are also retained: AT01–AT04 → P5-01/P5-02/P5-15; AT05–AT06 → v1.x growth; AT07 → P5-04 (qualification portion v1.x); AT08 → P5-06/P5-08 (growth barriers v1.x); AT09 → P5-09/P5-20 (full growth expiry v1.x); AT10 → P5-05/P5-14; AT11 → P5-09/P6-01/P5-20; AT12 → P5-17; AT13 → P5-13. Remaining AT examples follow the source's responsible W packages via the table above; they are not new M2 gate criteria. Full original conditions, not these routing summaries, control tests.

Additional attack-example routing: AT14 → P5-08/P5-10 recovery gating (growth invalidation v1.x); AT15 → P5-17/P5-18 evidence completeness (growth experiments v1.x); AT16 → P5-05/P5-15 typed maintenance APIs; AT17 → v1.x W23; AT18 → P5-08/X06; AT19 → v1.x W06/W07; AT20 → v1.x W07/W09/W10 and W00-C01; AT21 → v1.x W09/W10; AT22 → P5-01/P5-06/P5-18/SYS25; AT23 → P5-08/P5-09/I21.

All positive growth claims, real-time dwell experiments, multi-root qualification, nonzero G/C, remote failover and M7 functionality remain **v1.x / not_run**. Never accelerate residence time, change thresholds, use structural fixtures as deployment proof, or count text-only modulation as strict J success.

## Update — 2026-10-19 (ADR 0009, ADR 0010)

ADR 0010 revises the v1.0 scope: v1.0 includes every mechanism of the v2.0 core (`docs/echo-core/echo-core-v2.0-first-principles.md`) and growth is on from birth. W06–W10 are replaced by the v2.0 growth work (evidence law, recomputation, EXPERIMENT lineage) and move into v1.0; W23 stays out of scope. The W06–W10 rows, the G/C acceptance rows and the growth-related invariant rows above are kept as history. The card that re-plans acceptance (P5-18) re-routes them against v2.0 §§15 and 17; until then this plan is not authoritative for growth items.
