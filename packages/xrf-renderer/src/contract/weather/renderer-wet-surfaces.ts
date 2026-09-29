/**
 * What rain wets the level's surfaces with (`CBlender_rain`), by reference, keys of the weather's textures.
 */
export interface IRendererWetSurfaces {
  /** `s_water`, a volume of rippling normals a slice a moment, which the renderer fetches and decodes itself. */
  splash: string;
  /** `s_waterFall`, the normals of water running down walls. */
  flow: string;
}
