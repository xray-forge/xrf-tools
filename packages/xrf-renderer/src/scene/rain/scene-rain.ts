import { Nullable } from "@xrf/types";
import { BufferAttribute, BufferGeometry, Mesh, Scene } from "three/webgpu";

import { IRendererRain } from "#/contract/weather/renderer-rain";
import { IRendererRainDrop } from "#/contract/weather/renderer-rain-drop";
import { IRainSurface, toRainSplashSurface, toRainStreakSurface } from "#/material/rain-surface.tsl";
import { createSceneMesh, createSceneRoot } from "#/scene/object/scene-mesh";
import { ISceneRainStaging } from "#/scene/rain/scene-rain-staging";
import { getClearTexture } from "#/texture/placeholder-textures";
import { RendererTextures } from "#/texture/renderer-textures";
import { RAIN_STREAKS, RainUniforms } from "#/uniforms/rain-uniforms";
import { WeatherTextures } from "#/weather/weather-textures";

/** One streak's quad: across it from minus one to one, then from its tail to its head. */
const CORNERS: ReadonlyArray<number> = [-1, 0, 1, 0, -1, 1, 1, 1];

/** Its two triangles. */
const QUAD: ReadonlyArray<number> = [0, 1, 2, 2, 1, 3];

/** One draw of the rain: its mesh, its surface, and the texture key its sampler is bound to. */
interface IRainDraw {
  mesh: Mesh;
  surface: IRainSurface;
  key: string;
}

/** The rain built for a weather: the streaks, and the splashes where the weather has a model for them. */
interface IRainBuild {
  scene: Scene;
  draws: ReadonlyArray<IRainDraw>;
}

/**
 * The rain `dxRainRender` draws: every streak it can draw at once as one mesh, each quad placed by its streak's fall
 * on the GPU, and as many copies of the splash's model, each placed by its streak's landing. Built once a weather
 * names its rain, compiled in the compile lane, then drawn.
 */
export class SceneRain {
  private readonly textures: RendererTextures;
  private readonly rain: RainUniforms;
  /** What draws, once compiled. */
  private current: Nullable<IRainBuild> = null;
  /** What waits to compile. */
  private pending: Nullable<IRainBuild> = null;
  private isCompiling: boolean = false;

  /**
   * @param textures - Where the rain's textures are put.
   * @param rain - What its shaders read.
   */
  public constructor(textures: RendererTextures, rain: RainUniforms) {
    this.textures = textures;
    this.rain = rain;
  }

  /** What to draw, or null while nothing has compiled or no weather names rain. */
  public get drawn(): Nullable<Scene> {
    return this.current?.scene ?? null;
  }

  /**
   * @param rain - What the weather's rain is drawn with, or null for none.
   */
  public take(rain: Nullable<IRendererRain>): void {
    if (this.pending) {
      this.release(this.pending);
    }

    this.pending = rain ? this.build(rain) : null;

    if (!rain && this.current) {
      this.release(this.current);
      this.current = null;
    }
  }

  /**
   * @returns The build waiting to compile, handed over once, or null.
   */
  public takeStaged(): Nullable<ISceneRainStaging> {
    const build: Nullable<IRainBuild> = this.pending;

    if (!build || this.isCompiling) {
      return null;
    }

    this.isCompiling = true;

    return {
      abandon: (): void => this.settle(build, false),
      commit: (): void => this.settle(build, true),
      scene: build.scene,
    };
  }

  public dispose(): void {
    [this.current, this.pending].forEach((build: Nullable<IRainBuild>) => build && this.release(build));
    this.current = null;
    this.pending = null;
  }

  private settle(build: IRainBuild, isCompiled: boolean): void {
    this.isCompiling = false;

    // Taken again meanwhile, the build was let go already.
    if (!isCompiled || build !== this.pending) {
      return;
    }

    if (this.current) {
      this.release(this.current);
    }

    this.current = build;
    this.pending = null;
  }

  private build(rain: IRendererRain): IRainBuild {
    const scene: Scene = createSceneRoot();
    const draws: Array<IRainDraw> = [this.toDraw(createStreakGeometry(), toRainStreakSurface(this.rain), rain.streak)];

    if (rain.drop) {
      draws.push(this.toDraw(createSplashGeometry(rain.drop), toRainSplashSurface(this.rain), rain.drop.texture));
    }

    draws.forEach((draw: IRainDraw) => scene.add(draw.mesh));

    return { draws, scene };
  }

  private toDraw(geometry: BufferGeometry, surface: IRainSurface, reference: string): IRainDraw {
    const key: string = WeatherTextures.toKey(reference);

    this.textures.target(key, getClearTexture(), surface.texture);

    return { key, mesh: createSceneMesh(geometry, null, surface.material), surface };
  }

  private release(build: IRainBuild): void {
    for (const { mesh, surface, key } of build.draws) {
      this.textures.unbind(key, surface.texture);
      mesh.geometry.dispose();
      surface.material.dispose();
    }
  }
}

/** Every streak's quad, each vertex naming its streak and its corner. */
function createStreakGeometry(): BufferGeometry {
  const geometry: BufferGeometry = new BufferGeometry();
  const corners: Float32Array = new Float32Array(RAIN_STREAKS * 8);
  const streaks: Float32Array = new Float32Array(RAIN_STREAKS * 4);
  const indices: Uint32Array = new Uint32Array(RAIN_STREAKS * 6);

  for (let streak: number = 0; streak < RAIN_STREAKS; streak += 1) {
    corners.set(CORNERS, streak * 8);
    streaks.fill(streak, streak * 4, streak * 4 + 4);
    QUAD.forEach((corner: number, at: number) => {
      indices[streak * 6 + at] = streak * 4 + corner;
    });
  }

  // Three wants a position to draw by; the material places every vertex itself.
  geometry.setAttribute("position", new BufferAttribute(new Float32Array(RAIN_STREAKS * 12), 3));
  geometry.setAttribute("corner", new BufferAttribute(corners, 2));
  geometry.setAttribute("streak", new BufferAttribute(streaks, 1));
  geometry.setIndex(new BufferAttribute(indices, 1));

  return geometry;
}

/** A copy of the splash's model for every streak, in renderer space, each vertex naming its streak. */
function createSplashGeometry(drop: IRendererRainDrop): BufferGeometry {
  const geometry: BufferGeometry = new BufferGeometry();
  const count: number = drop.positions.length / 3;
  const positions: Float32Array = new Float32Array(RAIN_STREAKS * count * 3);
  const uvs: Float32Array = new Float32Array(RAIN_STREAKS * count * 2);
  const streaks: Float32Array = new Float32Array(RAIN_STREAKS * count);
  const indices: Uint32Array = new Uint32Array(RAIN_STREAKS * drop.indices.length);
  // Engine `z` negated into renderer space.
  const model: Array<number> = drop.positions.map((it: number, at: number) => (at % 3 === 2 ? -it : it));

  for (let streak: number = 0; streak < RAIN_STREAKS; streak += 1) {
    positions.set(model, streak * count * 3);
    uvs.set(drop.uvs, streak * count * 2);
    streaks.fill(streak, streak * count, (streak + 1) * count);
    drop.indices.forEach((vertex: number, at: number) => {
      indices[streak * drop.indices.length + at] = streak * count + vertex;
    });
  }

  geometry.setAttribute("position", new BufferAttribute(positions, 3));
  geometry.setAttribute("uv", new BufferAttribute(uvs, 2));
  geometry.setAttribute("streak", new BufferAttribute(streaks, 1));
  geometry.setIndex(new BufferAttribute(indices, 1));

  return geometry;
}
