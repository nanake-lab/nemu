import type { Lang } from '@nemu/contracts';
import type { LipSyncProfile, ProfileMFCC } from './types.js';

function createProfile(lang: Lang, mfccMap: Record<'A' | 'I' | 'U' | 'E' | 'O', number[]>): LipSyncProfile {
  const vowels: Array<'A' | 'I' | 'U' | 'E' | 'O'> = ['A', 'I', 'U', 'E', 'O'];
  const mfccs: ProfileMFCC[] = vowels.map((v) => ({
    name: v,
    mfccCalibrationDataList: [{ array: [...mfccMap[v]] }],
    values: [...mfccMap[v]],
  }));

  return {
    targetSampleRate: 16000,
    sampleCount: 512,
    melFilterBankChannels: 16,
    compareMethod: 1, // L2_NORM
    mfccNum: 12,
    mfccDataCount: 1,
    useStandardization: false,
    mfccs,
  };
}

export const DEFAULT_ZH_PROFILE: LipSyncProfile = createProfile('zh', {
  A: [12.5, 4.2, -3.1, -1.8, 0.9, -0.6, -0.4, 0.3, 0.1, -0.2, 0.0, 0.1],
  I: [11.8, -6.5, 3.8, -2.1, 1.2, 0.5, -0.8, -0.2, 0.3, -0.1, 0.1, -0.1],
  U: [10.9, 1.2, 4.9, 2.3, -1.1, -0.7, 0.2, 0.4, -0.3, 0.1, -0.2, 0.0],
  E: [12.1, -3.2, 1.1, -3.0, 0.4, 0.8, -0.5, 0.1, -0.3, 0.2, 0.0, -0.1],
  O: [11.4, 3.8, 2.5, 0.8, -1.9, -0.3, 0.5, -0.1, 0.2, -0.3, 0.1, 0.0],
});

export const DEFAULT_EN_PROFILE: LipSyncProfile = createProfile('en', {
  A: [12.2, 3.9, -2.8, -1.5, 0.7, -0.8, -0.2, 0.4, 0.0, -0.3, 0.1, 0.0],
  I: [11.5, -7.0, 4.2, -2.4, 1.5, 0.3, -0.9, -0.1, 0.4, -0.2, 0.0, -0.1],
  U: [10.6, 1.5, 5.2, 2.0, -1.4, -0.5, 0.1, 0.5, -0.4, 0.0, -0.1, 0.1],
  E: [11.9, -3.8, 1.5, -2.7, 0.6, 0.7, -0.6, 0.2, -0.2, 0.1, 0.1, -0.2],
  O: [11.2, 4.2, 2.1, 0.5, -2.1, -0.1, 0.6, -0.2, 0.1, -0.4, 0.2, -0.1],
});

export const DEFAULT_JA_PROFILE: LipSyncProfile = createProfile('ja', {
  A: [12.6, 4.5, -3.3, -2.0, 1.0, -0.5, -0.3, 0.2, 0.1, -0.1, 0.0, 0.1],
  I: [11.9, -6.2, 3.5, -1.9, 1.1, 0.6, -0.7, -0.3, 0.2, -0.1, 0.2, 0.0],
  U: [10.8, 0.8, 4.6, 2.5, -0.9, -0.8, 0.3, 0.3, -0.2, 0.1, -0.1, 0.1],
  E: [12.2, -2.9, 0.9, -3.2, 0.3, 0.9, -0.4, 0.0, -0.4, 0.3, -0.1, 0.0],
  O: [11.5, 3.5, 2.8, 1.0, -1.7, -0.4, 0.4, 0.0, 0.3, -0.2, 0.0, -0.1],
});

export const DEFAULT_PROFILES: Record<Lang, LipSyncProfile> = {
  zh: DEFAULT_ZH_PROFILE,
  en: DEFAULT_EN_PROFILE,
  ja: DEFAULT_JA_PROFILE,
};

export function getProfileForLang(
  lang: Lang,
  customProfiles?: Partial<Record<Lang, LipSyncProfile>>,
): LipSyncProfile {
  if (customProfiles?.[lang]) {
    return customProfiles[lang]!;
  }
  return DEFAULT_PROFILES[lang] ?? DEFAULT_ZH_PROFILE;
}
