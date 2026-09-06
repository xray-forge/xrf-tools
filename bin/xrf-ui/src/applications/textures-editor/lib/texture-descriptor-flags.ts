/**
 * Whether a flag bit is set in a descriptor's flag word.
 *
 * @param flags - The whole word as the descriptor stores it.
 * @param bit - The bit to read, as the backend's vocabulary reported it.
 * @returns Whether it is set.
 */
export function hasTextureFlag(flags: number, bit: number): boolean {
  return (flags & bit) !== 0;
}

/**
 * The flag word with one bit set or cleared, leaving every other bit alone.
 *
 * Every other bit includes the ones the SDK never named. A word a third-party tool wrote bits into keeps them, because
 * dropping a bit nobody here understands is exactly the silent rewrite this editor exists to avoid.
 *
 * @param flags - The whole word as the descriptor stores it.
 * @param bit - The bit to change.
 * @param isSet - What to set it to.
 * @returns The new word.
 */
export function withTextureFlag(flags: number, bit: number, isSet: boolean): number {
  return isSet ? flags | bit : flags & ~bit;
}
