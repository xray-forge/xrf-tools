import { Nullable } from "@xrf/types";
import {
  BufferAttribute,
  BufferGeometry,
  ComputeNode,
  DoubleSide,
  Mesh,
  PerspectiveCamera,
  Scene,
  StorageBufferNode,
  WebGPURenderer,
} from "three/webgpu";

import { IRendererGrassSettings } from "#/contract/renderer-features";
import { IRendererGrass, IRendererGrassModel } from "#/contract/scene/renderer-grass";
import { toGrassSurfaceShader } from "#/material/grass-surface.tsl";
import { MaterialSamplers } from "#/material/material-samplers";
import { SurfaceNodeMaterial } from "#/material/surface-node-material";
import { ISurfaceShader } from "#/material/surface-shader";
import {
  createGrassItemBuffers,
  createGrassLevelBuffers,
  IGrassItemBuffers,
  IGrassLevelBuffers,
  listGrassItemStorage,
  listGrassLevelStorage,
  TGrassBuffers,
  toGrassItemCapacity,
} from "#/scene/grass/grass-buffers";
import { createGrassPlanting, IGrassPlanting, toGrassItems, toGrassStarts } from "#/scene/grass/grass-planting.tsl";
import { ISceneGrassStaging } from "#/scene/grass/scene-grass-staging";
import { createSceneMesh, createSceneRoot } from "#/scene/object/scene-mesh";
import { RendererTextures } from "#/texture/renderer-textures";
import { GrassUniforms } from "#/uniforms/grass-uniforms";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";
import { STATIC_DRAW_ARGUMENT_BYTES } from "#/uniforms/static-draw-buffers";

/** What one model draws with. */
interface IGrassDraw {
  mesh: Mesh;
  material: SurfaceNodeMaterial;
  samplers: MaterialSamplers;
}

/** Item lists of one capacity, the passes planting into them, and the draws reading them. */
interface IGrassBuild {
  capacity: number;
  items: IGrassItemBuffers;
  passes: IGrassPlanting;
  /** The four passes, in the order a frame runs them. */
  dispatches: Array<ComputeNode>;
  draws: Array<IGrassDraw>;
  /** What the draws are in, drawn once they compiled. */
  scene: Scene;
}

/**
 * A level's grass on the GPU: what it is planted from, the passes planting it around the camera every frame, and a
 * draw a model, each drawing the tufts the planting sorted into its range. The item lists grow when a setting needs
 * more room than they hold; any other change of the settings is only what the passes are dispatched over. Lists of a
 * new capacity are staged for the renderer to compile off the frame while the ones before keep planting, so no frame
 * builds a draw's pipeline.
 */
export class SceneGrass {
  /** Where the camera stands and what the planting is set to, as the planting reads them. */
  public readonly uniforms: GrassUniforms = new GrassUniforms();

  private readonly textures: RendererTextures;
  private readonly rendererUniforms: RendererUniforms;
  private grass: Nullable<IRendererGrass> = null;
  private level: Nullable<IGrassLevelBuffers> = null;
  /** What plants and draws. */
  private current: Nullable<IGrassBuild> = null;
  /** What is built to replace it, waiting to compile or compiling. */
  private pending: Nullable<IGrassBuild> = null;
  /** Whether the renderer took the pending build to compile. */
  private isCompiling: boolean = false;

  public constructor(textures: RendererTextures, uniforms: RendererUniforms) {
    this.textures = textures;
    this.rendererUniforms = uniforms;
  }

  /** Whether there is grass to plant, and every texture it is dressed with is up. */
  public get isReady(): boolean {
    return (
      this.grass !== null &&
      this.grass.models.length > 0 &&
      this.grass.models.every(
        (model: IRendererGrassModel) =>
          !model.surface.textures.base || this.textures.isUploaded(model.surface.textures.base)
      )
    );
  }

  /**
   * @param grass - A level's grass, replacing any put before.
   */
  public put(grass: IRendererGrass): void {
    this.release();
    this.grass = grass;
    this.level = createGrassLevelBuffers(grass);
  }

  /** Lets the grass go. */
  public release(): void {
    this.disposeBuild(this.current);

    // A build compiling goes once its compile ends: three is still building it.
    if (!this.isCompiling) {
      this.disposeBuild(this.pending);
    }

    this.current = null;
    this.pending = null;
    this.isCompiling = false;

    if (this.level) {
      this.rendererUniforms.retirement.retire(listGrassLevelStorage(this.level));
    }

    this.level = null;
    this.grass = null;
  }

  /**
   * @returns The build waiting to compile, handed over once, or null.
   */
  public takeStaged(): Nullable<ISceneGrassStaging> {
    const build: Nullable<IGrassBuild> = this.pending;

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

  /**
   * Plants the frame's grass around where the view stands: the four passes, over as many slots as the settings cover.
   * Lists the settings outgrow are built again and staged.
   *
   * @param renderer - The renderer drawing.
   * @param view - The view's camera, unjittered, which the planting centres on and culls by.
   * @param settings - What the grass is set to.
   * @returns What to draw, or null while nothing has compiled yet.
   */
  public plant(renderer: WebGPURenderer, view: PerspectiveCamera, settings: IRendererGrassSettings): Nullable<Scene> {
    const { grass, level, uniforms } = this;

    if (!grass || !level) {
      return null;
    }

    uniforms.configure(settings);
    uniforms.follow(view, grass.sizeX, grass.sizeZ, grass.offsetX, grass.offsetZ);

    const needed: number = uniforms.slotCount * uniforms.candidateCount;
    const capacity: number = toGrassItemCapacity(needed, this.rendererUniforms.staticDraws.storageLimit);

    // One build waits at a time; settings outgrowing it meanwhile are built for once it is in.
    if ((!this.current || capacity > this.current.capacity) && !this.pending) {
      this.pending = this.build(grass, level, capacity);
    }

    const { current } = this;

    if (!current) {
      return null;
    }

    current.passes.plant.count = Math.max(uniforms.slotCount, 1);
    // Past the lists' room the planting drops what does not fit, so nothing past it is scattered.
    current.passes.scatter.count = Math.max(Math.min(needed, current.capacity), 1);
    renderer.compute(current.dispatches);

    return current.scene;
  }

  public dispose(): void {
    this.release();
  }

  /** A staged build's compile ended: compiled, it replaces the one before; abandoned, it waits to be taken again. */
  private settle(build: IGrassBuild, isCompiled: boolean): void {
    // Released while it compiled.
    if (build !== this.pending) {
      this.disposeBuild(build);

      return;
    }

    this.isCompiling = false;

    if (isCompiled) {
      this.disposeBuild(this.current);
      this.current = build;
      this.pending = null;
    }
  }

  /** Item lists as large as asked, and the passes and draws reading them. */
  private build(grass: IRendererGrass, level: IGrassLevelBuffers, capacity: number): IGrassBuild {
    const items: IGrassItemBuffers = createGrassItemBuffers(capacity);
    const buffers: TGrassBuffers = { ...level, ...items };
    const passes: IGrassPlanting = createGrassPlanting(
      buffers,
      this.uniforms,
      this.rendererUniforms.staticDraws.lod.discard
    );
    const sorted: StorageBufferNode<"vec4"> = toGrassItems(items);
    const starts: StorageBufferNode<"uint"> = toGrassStarts(level);
    const scene: Scene = createSceneRoot();
    const draws: Array<IGrassDraw> = grass.models.map((model: IRendererGrassModel, index: number) => {
      const samplers: MaterialSamplers = new MaterialSamplers(
        this.textures,
        this.rendererUniforms.settings.textureBias
      );
      const shader: ISurfaceShader = toGrassSurfaceShader(
        { height: model.height, items: sorted, start: starts.element(index), surface: model.surface },
        samplers,
        this.rendererUniforms
      );
      const material: SurfaceNodeMaterial = new SurfaceNodeMaterial(
        this.rendererUniforms.staticDraws,
        this.rendererUniforms.treeWind
      );
      const geometry: BufferGeometry = new BufferGeometry();

      material.fragmentNode = shader.fragmentNode ?? null;
      material.positionViewNode = shader.positionViewNode ?? null;
      // `CULL_NONE`, as `CDetailManager::Render` draws every tuft.
      material.side = DoubleSide;
      geometry.setAttribute("position", new BufferAttribute(model.positions, 3));
      geometry.setAttribute("uv", new BufferAttribute(model.uvs, 2));
      geometry.setIndex(new BufferAttribute(model.indices, 1));
      geometry.setIndirect(level.args, index * STATIC_DRAW_ARGUMENT_BYTES);

      const mesh: Mesh = createSceneMesh(geometry, null, material);

      scene.add(mesh);

      return { material, mesh, samplers };
    });

    return {
      capacity,
      dispatches: [passes.clear, passes.plant, passes.arrange, passes.scatter],
      draws,
      items,
      passes,
      scene,
    };
  }

  /** Takes down a build's item lists and what reads them, keeping what the grass is planted from. */
  private disposeBuild(build: Nullable<IGrassBuild>): void {
    if (!build) {
      return;
    }

    build.draws.forEach(({ mesh, material, samplers }: IGrassDraw) => {
      build.scene.remove(mesh);
      mesh.geometry.dispose();
      material.dispose();
      samplers.release();
    });
    this.rendererUniforms.retirement.retire(listGrassItemStorage(build.items));
    // Their capacity and model count are in their shaders, so each rebuild is four pipelines three keeps until told.
    build.dispatches.forEach((compute: ComputeNode) => compute.dispose());
  }
}
