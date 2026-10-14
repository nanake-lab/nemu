# Application contracts

Implements `docs/CONTRACTS.md` 1, 2, 3, 4.1 and 7. Wire fields are not extended.
Echo Core IDL messages remain absent from `src/generated/` until P5-01.

Import types and schemas from `@nemu/contracts`. Validate unknown input at each
process boundary with the relevant schema's `safeParse`/`parse` method. Objects
are strict: unknown fields, non-finite numbers and mismatched versions fail.
`StageWireMessage` and `ApplicationEvent` are correlated discriminated unions;
generic `StageMessage<T, P>` and `AppBusEvent<T, P>` represent the documented envelopes.

- `controlMessageSchema`: only control commands/events.
- `audioMessageSchema`: only `stage.speech.*` commands/events.
- `microphoneMessageSchema`: Int16 PCM, exactly 320 samples (20 ms, 16 kHz mono).
- `stage.event` accepts only the six forwarding types listed in CONTRACTS 2.3.
- `motionSlots` supplies all 30 slots and their performance/request permissions.
- `bodyReadoutSchema` validates values, but does not compute or persist readouts.

## Streaming parser

```ts
const parse = createPerformanceParser({ outputId, warn: logger.warn.bind(logger) });
for await (const sentence of parse(textChunks)) {
  // sentence.text contains no valid/unknown closed performance markers.
}
```

`textChunks` may be synchronous or asynchronous; consumption is lazy and upstream
cancellation/errors propagate. The caller supplies its `@nemu/log` warning sink;
the package does not write to console, files or authoritative storage. Warnings
contain fixed diagnostic text, never dialogue or raw markers. `@nemu/log` is still
a scaffold, so wiring the sink belongs to the consuming application's task.

Implementation details for the documented segmentation rules:

- Counts Unicode code points, preserving surrogate pairs split across chunks.
- First chunk cuts at 20 CJK characters or eight completed English words;
  shorter punctuated sentences are emitted as soon as the punctuation boundary
  is known. A punctuation run uses one-token lookahead to keep ellipses and
  consecutive marks together; English endings require following whitespace.
- Later chunks exceeding 40 characters / 25 words prefer the latest clause
  separator; without one, cut at the character/whole-word limit. Words are
  whitespace-delimited. No timer or chunk-size-dependent decisions are used.
- Han + kana versus Latin counts determine the dominant script; kana selects
  Japanese for CJK-majority text, otherwise Han selects Chinese. Ties and text
  without those scripts use English. Punctuation uses its own documented rule.
- The 48-character open-marker buffer includes `[[`. An overlong open fragment
  becomes literal text as required by 3.2.5; a short unfinished marker at EOF is
  discarded with a warning, rather than spoken.
- The first three valid cues per resulting sentence are retained. Cues follow
  their text offsets across length cuts. The last retained voice cue controls
  `ttsStyle`, otherwise the last retained emotion cue, otherwise `neutral`.
- Sentence ids are deterministic `${outputId}:${index}` with zero-based indices.
  They are not Echo Core event ids. No persistent parser state is created.

Run `pnpm test:contracts` from the repository root for fixtures and V8 coverage;
CI enforces `performance.ts` line coverage >= 90%.
