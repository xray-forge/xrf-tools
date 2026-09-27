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
import { createSceneMesh } from "#/scene/object/scene-mesh";
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

/**
 * A level's grass on the GPU: what it is planted from, the passes planting it around the camera every frame, and a
 * draw a model, each drawing the tufts the planting sorted into its range. The item lists grow when a setting needs
 * more room than they hold; any other change of the settings is only what the passes are dispatched over.
 */
export class SceneGrass {
  /** What the grass pass draws, a mesh a model. */
  public readonly scene: Scene = new Scene();
  /** Where the camera stands and what the planting is set to, as the planting reads them. */
  public readonly uniforms: GrassUniforms = new GrassUniforms();

  private readonly textures: RendererTextures;
  private readonly rendererUniforms: RendererUniforms;
  private grass: Nullable<IRendererGrass> = null;
  private level: Nullable<IGrassLevelBuffers> = null;
  private items: Nullable<IGrassItemBuffers> = null;
  private passes: Nullable<IGrassPlanting> = null;
  /** The four passes, in the order a frame runs them. */
  private dispatches: Array<ComputeNode> = [];
  private draws: Array<IGrassDraw> = [];

  public constructor(textures: RendererTextures, uniforms: RendererUniforms) {
    this.textures = textures;
    this.rendererUniforms = uniforms;
    this.scene.matrixWorldAutoUpdate = false;
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
    this.clearItems();

    if (this.level) {
      this.rendererUniforms.retirement.retire(listGrassLevelStorage(this.level));
    }

    this.level = null;
    this.grass = null;
  }

  /**
   * Plants the frame's grass around where the view stands: the four passes, over as many slots as the settings cover.
   *
   * @param renderer - The renderer drawing.
   * @param view - The view's camera, unjittered, which the planting centres on and culls by.
   * @param settings - What the grass is set to.
   */
  public plant(renderer: WebGPURenderer, view: PerspectiveCamera, settings: IRendererGrassSettings): void {
    const { grass, level, uniforms } = this;

    if (!grass || !level) {
      return;
    }

    uniforms.configure(settings);
    uniforms.follow(view, grass.sizeX, grass.sizeZ, grass.offsetX, grass.offsetZ);

    const needed: number = uniforms.slotCount * uniforms.candidateCount;
    const capacity: number = toGrassItemCapacity(needed, this.rendererUniforms.staticDraws.storageLimit);
    const passes: IGrassPlanting =
      this.passes && this.items && capacity <= this.items.capacity ? this.passes : this.build(grass, level, capacity);

    passes.plant.count = Math.max(uniforms.slotCount, 1);
    // Past the lists' room the planting drops what does not fit, so nothing past it is scattered.
    passes.scatter.count = Math.max(Math.min(needed, this.items?.capacity ?? capacity), 1);
    renderer.compute(this.dispatches);
  }

  public dispose(): void {
    this.release();
  }

  /** The item lists as large as asked, and the passes and draws reading them. */
  private build(grass: IRendererGrass, level: IGrassLevelBuffers, capacity: number): IGrassPlanting {
    this.clearItems();

    const items: IGrassItemBuffers = createGrassItemBuffers(capacity);
    const buffers: TGrassBuffers = { ...level, ...items };
    const passes: IGrassPlanting = createGrassPlanting(
      buffers,
      this.uniforms,
      this.rendererUniforms.staticDraws.lod.discard
    );
    const sorted: StorageBufferNode<"vec4"> = toGrassItems(items);
    const starts: StorageBufferNode<"uint"> = toGrassStarts(level);

    this.items = items;
    this.passes = passes;
    this.dispatches = [passes.clear, passes.plant, passes.arrange, passes.scatter];
    this.draws = grass.models.map((model: IRendererGrassModel, index: number) => {
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

      this.scene.add(mesh);

      return { material, mesh, samplers };
    });

    return passes;
  }

  /** Takes down the item lists and what reads them, keeping what the grass is planted from. */
  private clearItems(): void {
    this.draws.forEach(({ mesh, material, samplers }: IGrassDraw) => {
      this.scene.remove(mesh);
      mesh.geometry.dispose();
      material.dispose();
      samplers.release();
    });
    this.draws = [];

    if (this.items) {
      this.rendererUniforms.retirement.retire(listGrassItemStorage(this.items));
    }

    // Their capacity and model count are in their shaders, so each rebuild is four pipelines three keeps until told.
    if (this.passes) {
      const { clear, plant, arrange, scatter } = this.passes;

      [clear, plant, arrange, scatter].forEach((compute: ComputeNode) => compute.dispose());
    }

    this.items = null;
    this.passes = null;
  }
}
