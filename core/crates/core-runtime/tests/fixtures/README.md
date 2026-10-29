# Generated canonical fixtures

Each `.hex` file is the first matching `request.name` entry's
`expected.canonical` from `core/contracts/golden/legal.json`, at main
`02948dc11aa1b43dc0dfe67c9da2e61676427385`, with one trailing newline.
`tests/common` decodes these through the real generated schema, then creates
explicitly synthetic structural cases. No new IDL or golden vectors are defined.

These fixtures do not establish valid genesis, real semantic interpretations,
growth evidence, model behavior, or runtime Q/L/U roots. Storage tests use owned
temporary directories and a deliberately synthetic seed transaction, then check
the runtime adapter against actual W02 publication and recovery APIs. They are
not the card's gateway/fact-view rehearsal smoke acceptance.
