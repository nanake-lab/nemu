export interface MotionSlotDefinition {
  id: string;
  category: 'idle' | 'fidget' | 'speech_gesture' | 'emotion_reaction' | 'desktop' | 'state';
  isPerformable: boolean; // 可表演 (act)
  isRequestable: boolean; // 可请求 (身体动作)
  user: string;
}

export const MOTION_SLOTS: Record<string, MotionSlotDefinition> = {
  // 待机（4）
  idle_stand: { id: 'idle_stand', category: 'idle', isPerformable: false, isRequestable: false, user: '物理状态机' },
  idle_relaxed: { id: 'idle_relaxed', category: 'idle', isPerformable: false, isRequestable: false, user: '物理状态机' },
  idle_hands_behind: { id: 'idle_hands_behind', category: 'idle', isPerformable: false, isRequestable: false, user: '物理状态机' },
  idle_look_around: { id: 'idle_look_around', category: 'idle', isPerformable: false, isRequestable: false, user: '物理状态机' },

  // 待机小动作（6）
  fidget_stretch: { id: 'fidget_stretch', category: 'fidget', isPerformable: false, isRequestable: true, user: '待机反射、身体动作请求' },
  fidget_yawn: { id: 'fidget_yawn', category: 'fidget', isPerformable: false, isRequestable: true, user: '待机反射、身体动作请求' },
  fidget_tail_play: { id: 'fidget_tail_play', category: 'fidget', isPerformable: false, isRequestable: true, user: '待机反射、身体动作请求' },
  fidget_ear_scratch: { id: 'fidget_ear_scratch', category: 'fidget', isPerformable: false, isRequestable: true, user: '待机反射、身体动作请求' },
  fidget_hum: { id: 'fidget_hum', category: 'fidget', isPerformable: false, isRequestable: true, user: '待机反射、身体动作请求' },
  fidget_sway: { id: 'fidget_sway', category: 'fidget', isPerformable: false, isRequestable: true, user: '待机反射、身体动作请求' },

  // 说话手势（6）
  wave: { id: 'wave', category: 'speech_gesture', isPerformable: true, isRequestable: true, user: '表演提示、身体动作请求' },
  nod: { id: 'nod', category: 'speech_gesture', isPerformable: true, isRequestable: true, user: '表演提示、身体动作请求' },
  shake_head: { id: 'shake_head', category: 'speech_gesture', isPerformable: true, isRequestable: true, user: '表演提示、身体动作请求' },
  think: { id: 'think', category: 'speech_gesture', isPerformable: true, isRequestable: true, user: '表演提示、身体动作请求' },
  explain: { id: 'explain', category: 'speech_gesture', isPerformable: true, isRequestable: true, user: '表演提示、身体动作请求' },
  cheer: { id: 'cheer', category: 'speech_gesture', isPerformable: true, isRequestable: true, user: '表演提示、身体动作请求' },

  // 情绪反应（4）
  react_surprised: { id: 'react_surprised', category: 'emotion_reaction', isPerformable: true, isRequestable: true, user: '表演提示、反射' },
  react_happy_hop: { id: 'react_happy_hop', category: 'emotion_reaction', isPerformable: true, isRequestable: true, user: '表演提示、反射' },
  react_pout: { id: 'react_pout', category: 'emotion_reaction', isPerformable: true, isRequestable: true, user: '表演提示、反射' },
  react_sigh: { id: 'react_sigh', category: 'emotion_reaction', isPerformable: true, isRequestable: true, user: '表演提示、反射' },

  // 桌面（4）
  sit_edge_loop: { id: 'sit_edge_loop', category: 'desktop', isPerformable: false, isRequestable: false, user: '物理状态机' },
  sit_to_stand: { id: 'sit_to_stand', category: 'desktop', isPerformable: false, isRequestable: false, user: '物理状态机' },
  walk: { id: 'walk', category: 'desktop', isPerformable: false, isRequestable: false, user: '物理状态机' },
  dangle: { id: 'dangle', category: 'desktop', isPerformable: false, isRequestable: false, user: '物理状态机' },

  // 状态（6）
  sleep_loop: { id: 'sleep_loop', category: 'state', isPerformable: false, isRequestable: false, user: '物理状态机' },
  wake_up: { id: 'wake_up', category: 'state', isPerformable: false, isRequestable: false, user: '物理状态机' },
  working_loop: { id: 'working_loop', category: 'state', isPerformable: false, isRequestable: false, user: '物理状态机' },
  peek_loop: { id: 'peek_loop', category: 'state', isPerformable: false, isRequestable: false, user: '物理状态机' },
  fall_loop: { id: 'fall_loop', category: 'state', isPerformable: false, isRequestable: false, user: '物理状态机' },
  land: { id: 'land', category: 'state', isPerformable: false, isRequestable: false, user: '物理状态机' },
};

export function isPerformableMotion(motionId: string): boolean {
  return MOTION_SLOTS[motionId]?.isPerformable === true;
}

export function isRequestableMotion(motionId: string): boolean {
  return MOTION_SLOTS[motionId]?.isRequestable === true;
}

export function getSlotDefinition(motionId: string): MotionSlotDefinition | undefined {
  return MOTION_SLOTS[motionId];
}
