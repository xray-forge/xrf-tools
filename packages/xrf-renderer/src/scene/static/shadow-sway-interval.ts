/**
 * Seconds between a shadow's draws for what sways alone, at most 60 a second: the sway is slow, and each draw over the
 * trees costs a light face's passes or a cascade most of its frame.
 */
export const SHADOW_SWAY_INTERVAL: number = 1 / 60;
