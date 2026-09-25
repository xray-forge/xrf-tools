import { Nullable } from "@xrf/types";
import {
  BufferAttribute,
  BufferGeometry,
  DoubleSide,
  Mesh,
  Scene,
  StorageBufferAttribute,
  WebGPURenderer,
} from "three/webgpu";

import { IRendererGrass, IRendererGrassModel } from "#/contract/scene/renderer-grass";
import { destroyStorageAttribute } from "#/internals/renderer-backend";
import { toGrassSurfaceShader } from "#/material/grass-surface.tsl";
import { MaterialSamplers } from "#/material/material-samplers";
import { SurfaceNodeMaterial } from "#/material/surface-node-material";
import { ISurfaceShader } from "#/material/surface-shader";
import { createGrassBuffers, IGrassBuffers } from "#/scene/grass/grass-buffers";
import { createGrassPlanting, IGrassPlanting, toGrassItems } from "#/scene/grass/grass-planting.tsl";
import { createSceneMesh } from "#/scene/object/scene-mesh";
import { RendererTextures } from "#/texture/renderer-textures";
import { GrassUniforms } from "#/uniforms/grass-uniforms";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";
import { STATIC_DRAW_ARGUMENTS } from "#/uniforms/static-draw-buffers";

/** Bytes one draw's indirect arguments take. */
const ARGUMENT_BYTES: number = STATIC_DRAW_ARGUMENTS * 4;

/** What one model draws with. */
interface IGrassDraw {
  mesh: Mesh;
  material: SurfaceNodeMaterial;
  samplers: MaterialSamplers;
}

/**
 * A level's grass on the GPU: what it is planted from, the passes planting it around the camera every frame, and a
 * draw a model, each drawing the tufts the planting sorted into its range. Built at the size the planting needs and
 * built again when a setting needs more.
 */
export class SceneGrass {
  /** What the grass pass draws, a mesh a model. */
  public readonly scene: Scene = new Scene();

  private readonly textures: RendererTextures;
  private readonly uniforms: RendererUniforms;
  private grass: Nullable<IRendererGrass> = null;
  private buffers: Nullable<IGrassBuffers> = null;
  private planting: Nullable<IGrassPlanting> = null;
  private draws: Array<IGrassDraw> = [];
  /** What the planting was built for: the slots it plants and the items its lists hold. */
  private slotCount: number = 0;
  private capacity: number = 0;
  /** Buffers a rebuild replaced, freed once a renderer is at hand. */
  private retired: Array<StorageBufferAttribute> = [];

  public constructor(textures: RendererTextures, uniforms: RendererUniforms) {
    this.textures = textures;
    this.uniforms = uniforms;
    this.scene.matrixWorldAutoUpdate = false;
  }

  /** The grid the grass is planted over, or null for none. */
  public get grid(): Nullable<Pick<IRendererGrass, "offsetX" | "offsetZ" | "sizeX" | "sizeZ">> {
    return this.grass;
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
  }

  /** Lets the grass go. */
  public release(): void {
    this.clear();
    this.grass = null;
  }

  /**
   * Plants the frame's grass for where the camera stands: the four passes, after building what they need at the size
   * the settings ask for.
   *
   * @param renderer - The renderer drawing.
   * @param uniforms - Where the camera stands and what the planting is set to, current.
   */
  public plant(renderer: WebGPURenderer, uniforms: GrassUniforms): void {
    this.retired.forEach((attribute: StorageBufferAttribute) => destroyStorageAttribute(renderer, attribute));
    this.retired = [];

    if (!this.grass) {
      return;
    }

    const capacity: number = uniforms.slotCount * uniforms.candidateCount;

    if (!this.planting || uniforms.slotCount !== this.slotCount || capacity > this.capacity) {
      this.build(this.grass, uniforms, capacity);
    }

    const planting: IGrassPlanting = this.planting!;

    renderer.compute([planting.clear, planting.plant, planting.arrange, planting.scatter]);
  }

  public dispose(): void {
    this.release();
  }

  private build(grass: IRendererGrass, uniforms: GrassUniforms, capacity: number): void {
    this.clear();

    const buffers: IGrassBuffers = createGrassBuffers(grass, capacity);
    const items = toGrassItems(buffers, capacity);

    this.buffers = buffers;
    this.slotCount = uniforms.slotCount;
    this.capacity = capacity;
    this.planting = createGrassPlanting(buffers, uniforms, this.uniforms.staticDraws.lod.discard, capacity);
    this.draws = grass.models.map((model: IRendererGrassModel, index: number) => {
      const samplers: MaterialSamplers = new MaterialSamplers(this.textures);
      const shader: ISurfaceShader = toGrassSurfaceShader(
        { height: model.height, items, surface: model.surface },
        samplers,
        this.uniforms
      );
      const material: SurfaceNodeMaterial = new SurfaceNodeMaterial(this.uniforms.staticDraws, this.uniforms.wind);
      const geometry: BufferGeometry = new BufferGeometry();

      material.fragmentNode = shader.fragmentNode ?? null;
      material.positionViewNode = shader.positionViewNode ?? null;
      // `CULL_NONE`, as `CDetailManager::Render` draws every tuft.
      material.side = DoubleSide;
      geometry.setAttribute("position", new BufferAttribute(model.positions, 3));
      geometry.setAttribute("uv", new BufferAttribute(model.uvs, 2));
      geometry.setIndex(new BufferAttribute(model.indices, 1));
      geometry.setIndirect(buffers.args, index * ARGUMENT_BYTES);

      const mesh: Mesh = createSceneMesh(geometry, null, material);

      this.scene.add(mesh);

      return { material, mesh, samplers };
    });
  }

  /** Takes down what was built, keeping the grass it was built from. */
  private clear(): void {
    this.draws.forEach(({ mesh, material, samplers }: IGrassDraw) => {
      this.scene.remove(mesh);
      mesh.geometry.dispose();
      material.dispose();
      samplers.release();
    });
    this.draws = [];

    if (this.buffers) {
      const { counts, cursors, items, itemModels, sorted, grid, slots, bins, triangles, dither, models } = this.buffers;

      this.retired.push(counts, cursors, items, itemModels, sorted, grid, slots, bins, triangles, dither, models);
    }

    this.buffers = null;
    this.planting = null;
    this.slotCount = 0;
    this.capacity = 0;
  }
}
