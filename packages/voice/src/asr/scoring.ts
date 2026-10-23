import type { Lang } from '@nemu/contracts';
export function units(text: string, lang: Lang): string[] {
  const normalized = text
    .normalize('NFKC')
    .toLowerCase()
    .replace(/[\p{P}\p{S}]/gu, '')
    .trim();
  return lang === 'en'
    ? normalized.split(/\s+/u).filter(Boolean)
    : [...normalized.replace(/\s/gu, '')];
}
export function editDistance(reference: readonly string[], hypothesis: readonly string[]): number {
  let previous = Array.from({ length: hypothesis.length + 1 }, (_, i) => i);
  for (let i = 1; i <= reference.length; i++) {
    const row = [i];
    for (let j = 1; j <= hypothesis.length; j++)
      row.push(
        Math.min(
          (row[j - 1] ?? 0) + 1,
          (previous[j] ?? 0) + 1,
          (previous[j - 1] ?? 0) + (reference[i - 1] === hypothesis[j - 1] ? 0 : 1),
        ),
      );
    previous = row;
  }
  return previous[hypothesis.length] ?? reference.length;
}
