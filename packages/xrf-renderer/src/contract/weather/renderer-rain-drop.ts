/**
 * The splash a raindrop leaves where it lands (`dm\rain.dm`): its mesh, in engine space, and its texture.
 */
export interface IRendererRainDrop {
  /** The texture's reference, a key of the weather's textures. */
  texture: string;
  /** Three numbers a vertex. */
  positions: ReadonlyArray<number>;
  /** Two numbers a vertex. */
  uvs: ReadonlyArray<number>;
  /** A triangle list. */
  indices: ReadonlyArray<number>;
}
