import { readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

const original = JSON.parse(await readFile(new URL("../semantic-candidate.schema.json", import.meta.url), "utf8"));
const common = original.$defs.sourceMetadata.properties;
const definitions = Object.fromEntries(["refCandidate", "claimCandidate", "actCandidate"].map((name) => {
  const detail = original.$defs[name].allOf[1];
  const properties = structuredClone({ ...common, ...detail.properties });
  properties.sourceSpan = { type: "array", minItems: 2, maxItems: 2, items: { type: "integer", minimum: 0 } };
  delete properties.sourceRefs.uniqueItems;
  return [name, { type: "object", additionalProperties: false, properties, required: Object.keys(properties) }];
}));
const wire = {
  type: "object",
  additionalProperties: false,
  required: original.required,
  properties: {
    ...original.properties,
    nodes: { type: "array", items: { anyOf: Object.keys(definitions).map((name) => ({ $ref: `#/$defs/${name}` })) } }
  },
  $defs: definitions
};
const output = new URL("../semantic-candidate.wire.schema.json", import.meta.url);
await writeFile(output, `${JSON.stringify(wire, null, 2)}\n`, "utf8");
process.stdout.write(`wrote ${fileURLToPath(output)}\n`);
