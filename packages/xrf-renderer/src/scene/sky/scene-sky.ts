import { Nullable } from "@xrf/types";
import { Texture } from "three/webgpu";

import { IRendererSky } from "#/contract/renderer-sky";
import { getClearTexture, getPlaceholderSkyTexture } from "#/texture/placeholder-textures";
import { RendererTextures } from "#/texture/renderer-textures";
import { ITextureTarget } from "#/texture/texture-target";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";
import { SkyUniforms } from "#/uniforms/sky-uniforms";

/** The two skies, their irradiance cubes, then the two clouds textures. */
const SLOTS: number = 6;

/** Where the irradiance cubes start among the slots. */
const ENVIRONMENTS: number = 2;

/** The slots of each keyframe: its sky, its irradiance cube and its clouds. */
const SIDES: readonly [ReadonlyArray<number>, ReadonlyArray<number>] = [
  [0, 2, 4],
  [1, 3, 5],
];

/**
 * The two skies the lighting names, their irradiance cubes and their clouds, bound into what samples them by their
 * texture keys: each draws its placeholder until the consumer's texture is up, then the texture. A keyframe whose
 * textures are not all up is blended out, so a sky still going up shows the other one rather than its placeholder.
 */
export class SceneSky {
  private readonly textures: RendererTextures;
  private readonly sky: SkyUniforms;
  private readonly keys: Array<Nullable<string>> = new Array(SLOTS).fill(null);
  private readonly samplers: ReadonlyArray<ITextureTarget>;
  private readonly placeholders: ReadonlyArray<Texture>;
  /** How far from the first keyframe to the second the lighting asks for. */
  private blend: number = 0;

  /**
   * @param textures - Where the skies' textures are put.
   * @param uniforms - What samples them.
   */
  public constructor(textures: RendererTextures, uniforms: Pick<RendererUniforms, "sky" | "clouds">) {
    this.textures = textures;
    this.sky = uniforms.sky;
    this.samplers = [...uniforms.sky.cubes, ...uniforms.sky.environments, ...uniforms.clouds.textures];
    this.placeholders = [
      ...new Array<Texture>(4).fill(getPlaceholderSkyTexture()),
      getClearTexture(),
      getClearTexture(),
    ];
  }

  /**
   * @param sky - The skies the lighting names now.
   */
  public take(sky: IRendererSky): void {
    this.blend = sky.blend;

    const keys: ReadonlyArray<Nullable<string>> = [
      ...sky.textures,
      ...sky.environments,
      ...(sky.clouds?.textures ?? [null, null]),
    ];

    keys.forEach((key: Nullable<string>, index: number) => {
      const previous: Nullable<string> = this.keys[index];

      if (previous === key) {
        return;
      }

      if (previous) {
        this.textures.unbind(previous, this.samplers[index]);
      }

      if (key) {
        this.textures.target(key, this.placeholders[index], this.samplers[index]);
      } else {
        this.samplers[index].value = this.placeholders[index];
      }

      this.keys[index] = key;
    });
  }

  /**
   * Lights the hemisphere by the irradiance cubes once both are up, by the lighting's stand-in until then, and blends
   * out a keyframe still going up.
   */
  public update(): void {
    const [isFirstUp, isSecondUp] = SIDES.map((slots: ReadonlyArray<number>) =>
      slots.every((slot: number) => {
        const key: Nullable<string> = this.keys[slot];

        return key === null || this.textures.isUploaded(key);
      })
    );

    this.sky.blend.value = isFirstUp === isSecondUp ? this.blend : isSecondUp ? 1 : 0;

    const first: Nullable<string> = this.keys[ENVIRONMENTS];
    const second: Nullable<string> = this.keys[ENVIRONMENTS + 1];
    const isUp: boolean =
      first !== null &&
      second !== null &&
      this.textures.getUploaded(first) !== null &&
      this.textures.getUploaded(second) !== null;

    this.sky.environmentsUp.value = isUp ? 1 : 0;
  }

  public dispose(): void {
    this.keys.forEach((key: Nullable<string>, index: number) => {
      if (key) {
        this.textures.unbind(key, this.samplers[index]);
      }
    });
    this.keys.fill(null);
  }
}
