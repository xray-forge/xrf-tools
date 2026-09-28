import { Maybe, Nullable } from "@xrf/types";
import { Material, MeshBasicNodeMaterial, WebGPURenderer } from "three/webgpu";

import { ERendererDraw } from "#/contract/scene/renderer-draw";
import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { createOpaqueShadowMaterial, createSurfaceMaterial, ISurfaceMaterial } from "#/material/surface-material";
import { SurfacePrograms } from "#/material/surface-programs";
import { SurfaceBatching } from "#/scene/surface/surface-batching";
import { RendererTextures } from "#/texture/renderer-textures";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/** Superseded materials kept compiled, so a toggle back to one draws it at once. */
const MATERIAL_CACHE_LIMIT: number = 64;

/** What a wireframe draws every static surface's edges with: one untextured grey, lit as a surface is. */
const WIREFRAME_SURFACE: IRendererSurface = { color: [0.75, 0.75, 0.75], draw: ERendererDraw.OPAQUE, textures: {} };

/**
 * The surfaces a consumer put, by key, each as the material the frame draws it with.
 * A material no surface names any more is kept, compiled, for a surface put again as the same description.
 */
export class SurfaceLibrary {
  private readonly textures: RendererTextures;
  private readonly uniforms: RendererUniforms;
  private readonly onReplaced: (key: string, release: Nullable<() => void>) => void;

  private readonly materials: Map<string, ISurfaceMaterial> = new Map();
  /** The description each key's material was built from. */
  private readonly descriptions: Map<string, string> = new Map();
  /** The description each material was built from, which is the cache's key for it. */
  private readonly built: WeakMap<ISurfaceMaterial, string> = new WeakMap();
  /** Materials no surface names any more, disposed or cached once nothing draws them. */
  private readonly retired: Set<ISurfaceMaterial> = new Set();
  /** Compiled materials no surface names, by description, oldest first. */
  private readonly cache: Map<string, ISurfaceMaterial> = new Map();
  private isWireframe: boolean = false;
  /** What a wireframe draws the static surfaces' edges with, made the first time one draws. */
  private wireframe: Maybe<ISurfaceMaterial>;
  /** The shaders its materials share, one per variant. */
  private readonly programs: SurfacePrograms;
  /** The shadow material every opaque surface it makes shares. */
  private readonly opaqueShadow: MeshBasicNodeMaterial;
  /** Which of its surfaces a static batch draws by a material they share. */
  private readonly batching: SurfaceBatching;
  /** The key each material is put under, which its users are found by. */
  private readonly keyOf: Map<ISurfaceMaterial, string> = new Map();

  /**
   * @param textures - Where the materials bind their textures.
   * @param uniforms - What their shaders read.
   * @param onReplaced - Told when a key's material or its batched view changed, so whatever draws it can draw the new
   *   one, with what lets go of the view it drew by once that change applies, or null for none.
   * @param onInvalidated - Told a key as a texture's where what the shared materials bind under it was replaced, which
   *   bundles drawing them record again for.
   */
  public constructor(
    textures: RendererTextures,
    uniforms: RendererUniforms,
    onReplaced: (key: string, release: Nullable<() => void>) => void,
    onInvalidated: (key: string) => void
  ) {
    this.textures = textures;
    this.uniforms = uniforms;
    this.onReplaced = onReplaced;
    this.programs = new SurfacePrograms(uniforms);
    this.opaqueShadow = createOpaqueShadowMaterial(this.programs, uniforms);
    this.batching = new SurfaceBatching(textures, uniforms, this.programs, onInvalidated);
  }

  /** Layers the device allows a texture array, which the device says once it is open. */
  public set arrayLayerLimit(limit: number) {
    this.batching.layerLimit = limit;
  }

  /**
   * @param key - A texture's key whose samplers were pointed at another texture: every surface whose batched view
   *   changed with it is drawn again.
   */
  public rebind(key: string): void {
    // Only a material put under a key is tracked: one retired lets its view go with the change that retired it.
    for (const [material, release] of this.batching.rebind(key)) {
      this.onReplaced(this.keyOf.get(material) as string, release);
    }
  }

  /**
   * @param renderer - The renderer drawing, the shared materials' arrays and rows put on the GPU before it does.
   */
  public flush(renderer: WebGPURenderer): void {
    this.batching.flush(renderer);
  }

  /** Whether any material waits for nothing to draw it, a shared one no surface draws by among them. */
  public get hasRetired(): boolean {
    return this.retired.size > 0 || this.batching.hasIdle;
  }

  public get(key: string): Maybe<ISurfaceMaterial> {
    return this.materials.get(key);
  }

  /**
   * @param key - What the surface is put under.
   * @param surface - What it is; the same surface again keeps its material, since building one compiles its shader.
   */
  public put(key: string, surface: IRendererSurface): void {
    if (this.descriptions.get(key) !== this.toDescription(surface)) {
      this.build(key, surface);
    }
  }

  public release(key: string): void {
    const previous: Maybe<ISurfaceMaterial> = this.materials.get(key);

    this.materials.delete(key);
    this.descriptions.delete(key);
    this.onReplaced(key, previous ? this.retireMaterial(previous) : null);
  }

  /** What a wireframe draws every static surface's edges with, over the arenas' line indices. */
  public get wireframeMaterial(): ISurfaceMaterial {
    this.wireframe ??= createSurfaceMaterial(
      WIREFRAME_SURFACE,
      this.textures,
      this.uniforms,
      this.programs,
      this.opaqueShadow
    );

    return this.wireframe;
  }

  /**
   * Has every material draw its triangles' edges, or its triangles: what a part drawn plainly draws. Nothing is built
   * again, so nothing compiles but what draws plainly; the static draws take their own wireframe.
   *
   * @param isWireframe - Whether every surface draws as its triangles' edges.
   */
  public setWireframe(isWireframe: boolean): void {
    if (isWireframe === this.isWireframe) {
      return;
    }

    this.isWireframe = isWireframe;

    for (const surfaces of [this.materials.values(), this.retired, this.cache.values()]) {
      for (const surface of surfaces) {
        SurfaceLibrary.applyWireframe(surface, isWireframe);
      }
    }
  }

  /**
   * An impostor is never drawn plainly, and its wireframe batch draws the line index with its own material: three
   * would build a line index of its own over that one, as big as the arena, for a material marked wireframe.
   */
  private static applyWireframe(surface: ISurfaceMaterial, isWireframe: boolean): void {
    if (!surface.isImpostor && surface.material.wireframe !== isWireframe) {
      surface.material.wireframe = isWireframe;
      surface.material.needsUpdate = true;
    }
  }

  /**
   * Caches, or disposes past the cache's limit, every retired material nothing draws any more.
   *
   * @param drawn - Every material something draws.
   * @param isCompiled - Whether a material compiled, which makes it worth caching.
   */
  public retire(drawn: ReadonlySet<Material>, isCompiled: (material: Material) => boolean): void {
    for (const surface of this.retired) {
      if (drawn.has(surface.material)) {
        continue;
      }

      this.retired.delete(surface);

      const description: Maybe<string> = this.built.get(surface);

      if (description && isCompiled(surface.material) && !this.cache.has(description)) {
        this.cache.set(description, surface);
      } else {
        surface.dispose();
      }
    }

    this.batching.retire(drawn);

    while (this.cache.size > MATERIAL_CACHE_LIMIT) {
      const [description, surface] = this.cache.entries().next().value as [string, ISurfaceMaterial];

      this.cache.delete(description);
      surface.dispose();
    }
  }

  public dispose(): void {
    for (const surfaces of [this.materials.values(), this.retired, this.cache.values()]) {
      for (const surface of surfaces) {
        surface.dispose();
      }
    }

    this.wireframe?.dispose();
    this.wireframe = undefined;
    this.batching.dispose();
    this.opaqueShadow.dispose();
    this.programs.dispose();
    this.keyOf.clear();

    this.materials.clear();
    this.retired.clear();
    this.cache.clear();
    this.descriptions.clear();
  }

  /** A key's material for its description now, from the cache when one was compiled for it before. */
  private build(key: string, surface: IRendererSurface): void {
    const description: string = this.toDescription(surface);
    const previous: Maybe<ISurfaceMaterial> = this.materials.get(key);
    let material: Maybe<ISurfaceMaterial> = this.cache.get(description);

    if (material) {
      this.cache.delete(description);
    } else {
      material = createSurfaceMaterial(surface, this.textures, this.uniforms, this.programs, this.opaqueShadow);
      this.built.set(material, description);
    }

    SurfaceLibrary.applyWireframe(material, this.isWireframe);

    this.descriptions.set(key, description);
    this.materials.set(key, material);

    const release: Nullable<() => void> = previous ? this.retireMaterial(previous) : null;

    this.keyOf.set(material, key);
    this.batching.track(material, surface);
    this.onReplaced(key, release);
  }

  /**
   * Retires a material no key names any more, kept until nothing draws it; its batched view goes with the change that
   * rebuilds what drew it, which the view's static draws read until then.
   *
   * @returns What lets go of the view, or null for a material batched by none.
   */
  private retireMaterial(material: ISurfaceMaterial): Nullable<() => void> {
    this.keyOf.delete(material);
    this.retired.add(material);

    return this.batching.untrack(material);
  }

  /** What a surface is built from, as one comparable string. */
  private toDescription(surface: IRendererSurface): string {
    return JSON.stringify(surface);
  }
}
