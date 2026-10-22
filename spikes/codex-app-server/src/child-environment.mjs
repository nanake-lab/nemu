/** Remove the parent desktop session's capability transport and API overrides. */
export function isolatedChildEnvironment(environment, codeHome) {
  return {
    ...Object.fromEntries(Object.entries(environment).filter(([name]) =>
      !name.toUpperCase().startsWith("CODEX_") &&
      !["OPENAI_API_KEY", "OPENAI_BASE_URL"].includes(name.toUpperCase()))),
    CODEX_HOME: codeHome
  };
}
