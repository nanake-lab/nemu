import Ajv2020 from "ajv/dist/2020.js";
import { readFile } from "node:fs/promises";

const schema = JSON.parse(await readFile(new URL("../semantic-candidate.schema.json", import.meta.url), "utf8"));
const ajv = new Ajv2020({ strict: false, allErrors: true });
const validator = ajv.compile(schema);

export function validateSchema(candidate) {
  return {
    valid: validator(candidate),
    errors: (validator.errors ?? []).map(({ instancePath, keyword }) => ({ instancePath, keyword }))
  };
}
