import { mkdir, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const outPath = resolve(repoRoot, "spikes/codex-app-server/fixtures/irl-cases.json");
const templates = {
  zh: [
    ["plan", "我计划在{n}号整理书桌。"],
    ["request", "请在{n}号提醒我提交报告。"],
    ["uncertainty", "我不确定周末会不会下雨，概率大约是{n}成。"],
    ["conditional", "如果会议延长，我就改到{n}点吃饭。"],
    ["quotation", "小林说：「下周{n}号可能会下雪。」"],
    ["correction", "更正：我说的不是周二，是周{n}天。"],
    ["commitment", "我答应在{n}号前发送文件。"],
    ["question", "你能告诉我{n}乘二是多少吗？"],
    ["report", "我刚刚完成了第{n}个练习。"],
    ["preference", "我觉得第{n}首歌听起来很安静。"],
    ["negation", "我没有在{n}号收到包裹。"],
    ["hypothesis", "这只是第{n}个假设，不代表已经发生。"],
    ["request", "请把第{n}个蓝色文件夹放到桌面上。"],
    ["quotation", "我引用一句话：「第{n}步保持简单。」"],
    ["desire", "我希望在{n}号之前读完这本书。"],
    ["possibility", "也许第{n}场会议需要再检查数据。"],
    ["question", "小林问：「你会在{n}点回来吗？」"],
    ["correction", "纠正一下，清单上应该是第{n}项。"],
    ["conditional", "如果有空，我会读第{n}章。"],
    ["uncertainty", "我可能忘了更新第{n}份清单。"]
  ],
  en: [
    ["plan", "I plan to tidy the desk on day {n}."],
    ["request", "Please remind me to submit the report on day {n}."],
    ["uncertainty", "I am not sure whether it will rain this weekend; the chance is about {n} in ten."],
    ["conditional", "If the meeting runs long, I will move lunch to {n} o'clock."],
    ["quotation", "Lin said, ‘It may snow on day {n} next week.’"],
    ["correction", "Correction: I said Wednesday, not Tuesday; the date is day {n}."],
    ["commitment", "I promise to send the file before day {n}."],
    ["question", "Can you tell me what {n} times two is?"],
    ["report", "I just finished exercise number {n}."],
    ["preference", "I think song number {n} sounds calm."],
    ["negation", "I did not receive the parcel on day {n}."],
    ["hypothesis", "This is only hypothesis number {n}, not an event that happened."],
    ["request", "Please put the blue folder numbered {n} on the desk."],
    ["quotation", "I am quoting this sentence: ‘Keep step {n} simple.’"],
    ["desire", "I hope to finish this book before day {n}."],
    ["possibility", "Maybe we should check the data for meeting {n} again."],
    ["question", "Lin asked, ‘Will you return at {n} o'clock?’"],
    ["correction", "To correct the list, the item should be number {n}."],
    ["conditional", "If I have time, I will read chapter {n}."],
    ["uncertainty", "I may have forgotten to update checklist {n}."]
  ],
  ja: [
    ["plan", "{n}日に机を片付ける予定です。"],
    ["request", "{n}日に報告書を提出するよう、私にリマインドしてください。"],
    ["uncertainty", "週末に雨が降るか分かりません。確率は約{n}割です。"],
    ["conditional", "会議が延びたら、昼食を{n}時に変更します。"],
    ["quotation", "リンは「来週の{n}日に雪が降るかもしれない」と言いました。"],
    ["correction", "訂正します。火曜日ではなく水曜日で、日付は{n}日です。"],
    ["commitment", "{n}日までにファイルを送ると約束します。"],
    ["question", "{n}かける2はいくつか教えてくれますか。"],
    ["report", "練習の{n}番目を今終えました。"],
    ["preference", "{n}曲目は落ち着いた雰囲気だと思います。"],
    ["negation", "{n}日に小包を受け取りませんでした。"],
    ["hypothesis", "これは仮説の{n}番目で、実際に起きたことではありません。"],
    ["request", "番号{n}の青いフォルダーを机に置いてください。"],
    ["quotation", "「{n}番目の手順は簡潔に」と引用しています。"],
    ["desire", "{n}日までにこの本を読み終えたいです。"],
    ["possibility", "{n}回目の会議のデータをもう一度確認した方がよいかもしれません。"],
    ["question", "リンは「{n}時に戻りますか」と尋ねました。"],
    ["correction", "リストを訂正すると、項目は{n}番目です。"],
    ["conditional", "時間があれば、第{n}章を読みます。"],
    ["uncertainty", "チェックリスト{n}の更新を忘れたかもしれません。"]
  ]
};

const cases = [];
const languages = ["zh", "en", "ja"];
for (let index = 0; index < 200; index += 1) {
  const lang = languages[index % languages.length];
  const ordinal = Math.floor(index / languages.length) + 1;
  const [category, template] = templates[lang][(ordinal - 1) % templates[lang].length];
  const serial = String(index + 1).padStart(4, "0");
  cases.push({
    id: `case-${serial}`,
    sourceRef: `p0-08-${serial}`,
    lang,
    category,
    text: template.replaceAll("{n}", String(ordinal)),
    synthetic: true
  });
}

await mkdir(dirname(outPath), { recursive: true });
await writeFile(outPath, `${JSON.stringify({ dataset: "p0-08-synthetic-v1", count: cases.length, cases }, null, 2)}\n`, "utf8");
process.stdout.write(`wrote ${cases.length} synthetic cases to ${outPath}\n`);
