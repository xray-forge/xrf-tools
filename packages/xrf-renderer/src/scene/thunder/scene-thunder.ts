import { Maybe, Nullable } from "@xrf/types";
import { BufferAttribute, BufferGeometry, Matrix4, Mesh, Scene } from "three/webgpu";

import { ERendererDraw } from "#/contract/scene/renderer-draw";
import { IRendererThunder } from "#/contract/weather/renderer-thunder";
import { IRendererThunderbolt } from "#/contract/weather/renderer-thunderbolt";
import { IRendererThunderboltGradient } from "#/contract/weather/renderer-thunderbolt-gradient";
import { IRendererThunderboltModel } from "#/contract/weather/renderer-thunderbolt-model";
import { IRendererThunderboltStrike } from "#/contract/weather/renderer-thunderbolt-strike";
import { IThunderSurface, toThunderboltSurface, toThunderGlowSurface } from "#/material/thunder-surface.tsl";
import { createSceneMesh, createSceneRoot } from "#/scene/object/scene-mesh";
import { isSameDefinition } from "#/scene/same-definition";
import { ISceneBuildStaging } from "#/scene/staging/scene-build-staging";
import { StagedBuilds } from "#/scene/staging/staged-builds";
import { getClearTexture } from "#/texture/placeholder-textures";
import { RendererTextures } from "#/texture/renderer-textures";
import { ThunderUniforms } from "#/uniforms/thunder-uniforms";
import { WeatherTextures } from "#/weather/weather-textures";

/** A glow's quad: its corners across and up, from minus one to one. */
const CORNERS: ReadonlyArray<number> = [1, 1, 1, -1, -1, 1, -1, -1];

/** Its two triangles. */
const QUAD: ReadonlyArray<number> = [0, 1, 2, 2, 1, 3];

/** Which of a bolt's glows a draw is. */
type TThunderGlow = "top" | "center";

/** One draw of the thunder: its mesh, its surface, and the texture key its sampler is bound to. */
interface IThunderDraw {
  mesh: Mesh;
  surface: IThunderSurface;
  key: string;
}

/** The thunder built for a weather: a draw for every model, and one for every glow the bolts draw. */
interface IThunderBuild {
  scene: Scene;
  thunder: IRendererThunder;
  /** By model index. */
  models: ReadonlyArray<IThunderDraw>;
  /** By `toGlowKey`. */
  glows: ReadonlyMap<string, IThunderDraw>;
}

/**
 * The bolts `dxThunderboltRender` draws: every model the weather's bolts name and every glow they draw, built once a
 * weather names its thunder and compiled in the compile lane, then shown one bolt at a time as it strikes.
 */
export class SceneThunder {
  private readonly textures: RendererTextures;
  private readonly thunder: ThunderUniforms;
  private readonly builds: StagedBuilds<IThunderBuild> = new StagedBuilds({
    onCommit: (build: IThunderBuild) => this.show(build, this.shown),
    release: (build: IThunderBuild) => this.release(build),
  });
  /** What was last taken, which a weather sent again unchanged keeps built. */
  private taken: Nullable<IRendererThunder> = null;
  /** The bolt striking now, shown on whatever build draws. */
  private shown: Nullable<IRendererThunderboltStrike> = null;
  private readonly matrix: Matrix4 = new Matrix4();

  /**
   * @param textures - Where the thunder's textures are put.
   * @param thunder - What its shaders read.
   */
  public constructor(textures: RendererTextures, thunder: ThunderUniforms) {
    this.textures = textures;
    this.thunder = thunder;
  }

  /** What to draw, or null while no bolt strikes or nothing has compiled. */
  public get drawn(): Nullable<Scene> {
    return this.shown ? (this.builds.current?.scene ?? null) : null;
  }

  /**
   * @param thunder - What the weather strikes with, or null for none.
   */
  public take(thunder: Nullable<IRendererThunder>): void {
    // A keyframe edited by hand sends its weather again, the bolts as they were: nothing is built again for them.
    if (isSameDefinition(thunder, this.taken)) {
      return;
    }

    this.taken = thunder;

    if (thunder) {
      this.builds.stage(this.build(thunder));
    } else {
      this.builds.clear();
    }
  }

  /**
   * @param strike - The bolt striking this frame, or null for none.
   */
  public strike(strike: Nullable<IRendererThunderboltStrike>): void {
    this.shown = strike;

    if (this.builds.current) {
      this.show(this.builds.current, strike);
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

  /** Hides every draw but the striking bolt's model and glows, its model placed where it strikes. */
  private show(build: IThunderBuild, strike: Nullable<IRendererThunderboltStrike>): void {
    const bolt: Maybe<IRendererThunderbolt> = strike ? build.thunder.bolts[strike.bolt] : undefined;

    build.models.forEach((draw: IThunderDraw) => (draw.mesh.visible = false));
    build.glows.forEach((draw: IThunderDraw) => (draw.mesh.visible = false));

    if (!strike || !bolt) {
      return;
    }

    const model: Maybe<IThunderDraw> = bolt.model === null ? undefined : build.models[bolt.model];

    if (model) {
      this.place(model.mesh, strike);
      model.mesh.visible = true;
    }

    for (const glow of ["top", "center"] as const) {
      const draw: Maybe<IThunderDraw> = build.glows.get(toGlowKey(glow, bolt[glow]));

      if (draw) {
        draw.mesh.visible = true;
      }
    }
  }

  /** `current_xform`: the model's axes as the strike points them, from where it strikes. */
  private place(mesh: Mesh, strike: IRendererThunderboltStrike): void {
    const [i, j, k] = strike.axes;
    const [x, y, z] = strike.position;

    this.matrix.set(i[0], j[0], k[0], x, i[1], j[1], k[1], y, i[2], j[2], k[2], z, 0, 0, 0, 1);
    mesh.matrix.copy(this.matrix);
    mesh.matrixWorld.copy(this.matrix);
  }

  private build(thunder: IRendererThunder): IThunderBuild {
    const scene: Scene = createSceneRoot();
    const models: Array<IThunderDraw> = thunder.models.map((model: IRendererThunderboltModel) =>
      this.toDraw(createModelGeometry(model), toThunderboltSurface(model.draw, this.thunder), model.texture, 0)
    );
    const glows: Map<string, IThunderDraw> = new Map();

    for (const bolt of Object.values(thunder.bolts)) {
      for (const [glow, order] of [
        ["top", 1],
        ["center", 2],
      ] as const) {
        const gradient: IRendererThunderboltGradient = bolt[glow];
        const key: string = toGlowKey(glow, gradient);

        if (!glows.has(key)) {
          glows.set(
            key,
            this.toDraw(
              createGlowGeometry(),
              toThunderGlowSurface(gradient.draw, this.thunder[glow]),
              gradient.texture,
              order
            )
          );
        }
      }
    }

    [...models, ...glows.values()].forEach((draw: IThunderDraw) => scene.add(draw.mesh));

    return { glows, models, scene, thunder };
  }

  private toDraw(geometry: BufferGeometry, surface: IThunderSurface, reference: string, order: number): IThunderDraw {
    const key: string = WeatherTextures.toKey(reference);
    const mesh: Mesh = createSceneMesh(geometry, null, surface.material);

    this.textures.target(key, getClearTexture(), surface.texture);
    // The model first, then its top glow and its middle one, as the engine draws them.
    mesh.renderOrder = order;

    return { key, mesh, surface };
  }

  private release(build: IThunderBuild): void {
    for (const { mesh, surface, key } of [...build.models, ...build.glows.values()]) {
      this.textures.unbind(key, surface.texture);
      mesh.geometry.dispose();
      surface.material.dispose();
    }
  }
}

/** What tells one glow draw from another: which glow it is, how it composites, and its texture. */
function toGlowKey(glow: TThunderGlow, gradient: { draw: ERendererDraw; texture: string }): string {
  return `${glow}:${gradient.draw}:${gradient.texture}`;
}

/** A bolt's model in renderer space, its length of one down its second axis. */
function createModelGeometry(model: IRendererThunderboltModel): BufferGeometry {
  const geometry: BufferGeometry = new BufferGeometry();

  // Engine `z` negated into renderer space.
  geometry.setAttribute(
    "position",
    new BufferAttribute(
      Float32Array.from(model.positions, (it: number, at: number) => (at % 3 === 2 ? -it : it)),
      3
    )
  );
  geometry.setAttribute("uv", new BufferAttribute(Float32Array.from(model.uvs), 2));
  geometry.setIndex(new BufferAttribute(Uint32Array.from(model.indices), 1));

  return geometry;
}

/** A glow's quad, each vertex naming its corner; the material places it. */
function createGlowGeometry(): BufferGeometry {
  const geometry: BufferGeometry = new BufferGeometry();

  // Three wants a position to draw by; the material places every vertex itself.
  geometry.setAttribute("position", new BufferAttribute(new Float32Array(12), 3));
  geometry.setAttribute("corner", new BufferAttribute(Float32Array.from(CORNERS), 2));
  geometry.setIndex(new BufferAttribute(Uint32Array.from(QUAD), 1));

  return geometry;
}
