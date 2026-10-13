// Auto-generated from the renderer's contract by `cargo test -p xrf-renderer`. Do not edit it manually.

/** Cascades the sun's shadow can be cut into at most: the sun reads their texels from one `vec4`. */
export const RENDER_MAX_SHADOW_CASCADES: number = 4;

/** Texels the sun's shadow filter reaches each way at most: the widest the sun's pass has a pipeline for. */
export const RENDER_MAX_SHADOW_FILTER: number = 3;

/** Local lights a pixel marches contact shadows towards at most: the lights' shader keeps their weights in an array. */
export const RENDER_MAX_CONTACT_SHADOW_LIGHTS: number = 8;

/** Frames VBAO accumulates over at most. */
export const RENDER_MAX_AMBIENT_OCCLUSION_ACCUMULATION: number = 32;

/** GPU milliseconds a frame gives the work drawn once in so many frames that may wait: the renderer's default. */
export const DEFAULT_PERIODIC_BUDGET: number = 0.5;
