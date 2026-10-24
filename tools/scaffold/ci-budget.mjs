// Conservative estimate, not account billing telemetry. Configure the allowance for the actual plan.
const runs = Number(process.env.CI_MONTHLY_RUNS ?? 60);
const minutes = Number(process.env.CI_HOSTED_MINUTES_PER_RUN ?? 20);
const allowance = Number(process.env.CI_FREE_MINUTES ?? 2000);
if (![runs, minutes, allowance].every(Number.isFinite) || runs < 0 || minutes < 0 || allowance <= 0)
  throw new Error('Invalid CI budget settings');
const estimated = runs * minutes;
console.log(JSON.stringify({ runs, minutesPerRun: minutes, estimated, allowance }));
if (estimated > allowance * 0.8)
  console.log(
    `::warning::Estimated monthly hosted usage ${estimated} exceeds 80% of ${allowance} included minutes. Check account billing for actual usage.`,
  );
