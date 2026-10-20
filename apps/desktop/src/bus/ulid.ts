import { randomBytes } from 'node:crypto';

const ENCODING = '0123456789ABCDEFGHJKMNPQRSTVWXYZ';
const ENCODING_LEN = ENCODING.length;

export function generateUlid(timestamp: number = Date.now()): string {
  let timeStr = '';
  let time = timestamp;
  for (let i = 9; i >= 0; i--) {
    const mod = time % ENCODING_LEN;
    timeStr = ENCODING[mod] + timeStr;
    time = Math.floor(time / ENCODING_LEN);
  }

  const bytes = randomBytes(10);
  let randStr = '';
  for (let i = 0; i < 16; i++) {
    // Pick 5 bits from random bytes
    const byteIndex = Math.floor((i * 5) / 8);
    const bitOffset = (i * 5) % 8;
    let val = bytes[byteIndex] ?? 0;
    if (bitOffset > 3 && byteIndex + 1 < bytes.length) {
      val = (val << 8) | (bytes[byteIndex + 1] ?? 0);
      val = (val >> (11 - bitOffset)) & 0x1f;
    } else {
      val = (val >> (3 - bitOffset)) & 0x1f;
    }
    randStr += ENCODING[val];
  }

  return timeStr + randStr;
}
