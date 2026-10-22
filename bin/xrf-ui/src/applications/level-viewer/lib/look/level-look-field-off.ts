/** What leaves one of the look's sliders off: the physical light, or only its real-world light levels. */
export enum ELevelLookFieldOff {
  /** The engine's own tonemap, which the physical light's display transform replaces. */
  PHYSICAL_LIGHT = "physicalLight",
  /** The game's sun and flat ambient, which the physical light keeps at its authored levels only. */
  PHYSICAL_LEVELS = "physicalLevels",
}
