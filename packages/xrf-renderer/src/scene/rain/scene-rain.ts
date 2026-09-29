import { Nullable } from "@xrf/types";
import { BufferAttribute, BufferGeometry, Mesh, Scene } from "three/webgpu";

import { IRendererRain } from "#/contract/weather/renderer-rain";
import { IRendererRainDrop } from "#/contract/weather/renderer-rain-drop";
import { IRainSurface, toRainSplashSurface, toRainStreakSurface } from "#/material/rain-surface.tsl";
import { createSceneMesh, createSceneRoot } from "#/scene/object/scene-mesh";
import { isSameDefinition } from "#/scene/same-definition";
import { ISceneBuildStaging } from "#/scene/staging/scene-build-staging";
import { StagedBuilds } from "#/scene/staging/staged-builds";
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
  private readonly builds: StagedBuilds<IRainBuild> = new StagedBuilds({
    release: (build: IRainBuild) => this.release(build),
  });
  /** What was last taken, which a weather sent again unchanged keeps built. */
  private taken: Nullable<IRendererRain> = null;

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
    return this.builds.current?.scene ?? null;
  }

  /**
   * @param rain - What the weather's rain is drawn with, or null for none.
   */
  public take(rain: Nullable<IRendererRain>): void {
    // A keyframe edited by hand sends its weather again, the rain as it was: nothing is built again for it.
    if (isSameDefinition(rain, this.taken)) {
      return;
    }

    this.taken = rain;

    if (rain) {
      this.builds.stage(this.build(rain));
    } else {
      this.builds.clear();
    }
  }

  /**
   * @returns The build waiting to compile, handed over once, or null.
   */
  public takeStaged(): Nullable<ISceneBuildStaging> {
    return this.builds.takeStaged();
  }

  public dispose(): void {
    this.builds.clear();
    this.taken = null;
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
