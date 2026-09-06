/**
 * The gloss the SDK calls too dark to be worth a specular response.
 *
 * Mirrors `GenerateBumpResult::MINIMUM_GLOSS_POWER` in `xrf-texture`, which is what actually decides the
 * `isGlossTooDark` a run answers with. The number is here only to say it out loud - in the warning under the slider,
 * and in the notice after a run - and both have to agree with the backend, or a surface reports a threshold that
 * nothing was measured against.
 */
export const MINIMUM_GLOSS_POWER: number = 0.1;

/**
 * The level the generator's own field starts at.
 *
 * Comfortably above the threshold: a matte surface is a deliberate act rather than the state somebody should arrive
 * in, so the form opens on a gloss that does something and lets it be turned down.
 */
export const DEFAULT_GLOSS_POWER: number = 0.5;
