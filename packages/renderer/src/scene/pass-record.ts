import { ERendererPass } from "#/contract/scene/renderer-surface";

/** One of something for every pass that draws the consumer's scene. */
export type TPassRecord<T> = Record<ERendererPass, T>;

/** Every pass that draws the consumer's scene, in frame order. */
export const RENDERER_PASSES: ReadonlyArray<ERendererPass> = [
  ERendererPass.DEFERRED,
  ERendererPass.WALLMARK,
  ERendererPass.FORWARD,
];

/**
 * @param create - What each pass is given.
 * @returns One for every pass.
 */
export function toPassRecord<T>(create: (pass: ERendererPass) => T): TPassRecord<T> {
  return {
    [ERendererPass.DEFERRED]: create(ERendererPass.DEFERRED),
    [ERendererPass.FORWARD]: create(ERendererPass.FORWARD),
    [ERendererPass.WALLMARK]: create(ERendererPass.WALLMARK),
  };
}
