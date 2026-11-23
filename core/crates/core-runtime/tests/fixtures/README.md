# Generated canonical fixtures

Except for the successor fixture below, each `.hex` file is the first matching `request.name` entry's
`expected.canonical` from `core/contracts/golden/legal.json`, at main
`02948dc11aa1b43dc0dfe67c9da2e61676427385`, with one trailing newline.
`tests/common` decodes these through the real generated schema, then creates
explicitly synthetic structural cases. No new IDL or golden vectors are defined.

`ThetaSnapshotV2.hex` is the `ThetaSnapshotV2` entry with `event_count = 17`
from `core/contracts/golden/legal.json` at main
`67cc296c8b960ae429aceb1f1e0110ef0dbc2f9e`. It contains the complete state of
the existing synthetic C trajectory; runtime tests preserve it without filling
missing statistics. The old `ThetaSnapshot.hex` is retained only for rejecting
legacy writes. No fixture selects a production law or personality preset.

These fixtures do not establish valid genesis, real semantic interpretations,
growth evidence, model behavior, or runtime Q/L/U roots. Storage tests use owned
temporary directories and a deliberately synthetic seed transaction, then check
the runtime adapter against actual W02 publication and recovery APIs. They are
not the card's gateway/fact-view rehearsal smoke acceptance.
