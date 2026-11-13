// Exact screen function from PR #186, 3c530f87227ff54447717cb4a6e6031a299bae67.
export function screen(text, capped) {
  const letters = [...text.matchAll(/\p{L}/gu)].length;
  const han = [...text.matchAll(/\p{Script=Han}/gu)].length;
  return capped || !text.trim() || [...text].length > 300 || han < 6 || han / Math.max(1, letters) < 0.7
    || /[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f\ufffd]/u.test(text)
    || /(.{2,12})\1\1/u.test(text)
    || /<\||\[\[|<\/?think>|联结|共情|好奇|成事|注入|力度|steering|layer|alpha/iu.test(text);
}

