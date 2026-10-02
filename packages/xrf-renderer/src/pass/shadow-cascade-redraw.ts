import { SHADOW_SWAY_INTERVAL } from "#/scene/static/shadow-sway-interval";

/**
 * What of the sway interval a frame may come early by and still draw for the sway: at a 60 Hz display a frame a hair
 * short of an interval would otherwise skip it, halving the sway to every other frame.
 */
const SWAY_SLACK: number = 0.1;

/** Why a cascade due this frame may draw again. */
export interface IShadowCascadeRedraw {
  /** Its box moved, or anything it casts from changed. */
  isChanged: boolean;
  /** A caster in it moves, as a skinned part does. */
  isMoving: boolean;
  /** A caster in it sways with the wind blowing. */
  isSwaying: boolean;
  /** Seconds since it was last drawn. */
  sinceDrawn: number;
}

/**
 * @param redraw - Why the cascade may draw.
 * @returns Whether it draws: at once for a change or a moving caster, and for a swaying one once a sway interval passed.
 */
export function isShadowCascadeRedrawn(redraw: IShadowCascadeRedraw): boolean {
  return (
    redraw.isChanged ||
    redraw.isMoving ||
    (redraw.isSwaying && redraw.sinceDrawn >= SHADOW_SWAY_INTERVAL * (1 - SWAY_SLACK))
  );
}
