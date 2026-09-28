/** How a caster moves while it casts, which has a kept shadow over it drawn again. */
export enum EShadowCasterMotion {
  /** Not at all: a shadow over it is drawn again only once something there changes. */
  STILL = 0,
  /** With the wind: while it blows. */
  SWAYING = 1,
  /** On its own, as a skinned part plays: always. */
  MOVING = 2,
}
