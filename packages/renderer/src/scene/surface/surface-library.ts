import { Maybe } from "@xrf/types";
import { Material } from "three/webgpu";

import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { createSurfaceMaterial, ISurfaceMaterial } from "#/material/surface-material";
import { RendererTextures } from "#/texture/renderer-textures";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/** Superseded materials kept compiled, so a toggle back to one draws it at once. */
const MATERIAL_CACHE_LIMIT: number = 64;

/**
 * The surfaces a consumer put, by key, each as the material the frame draws it with.
 * A material no surface names any more is kept, compiled, for a surface put again as the same description.
 */
export class SurfaceLibrary {
  private readonly textures: RendererTextures;
  private readonly uniforms: RendererUniforms;
  private readonly onReplaced: (key: string) => void;

  private readonly materials: Map<string, ISurfaceMaterial> = new Map();
  /** What each surface was put as, to build it again when the wireframe setting changes. */
  private readonly sources: Map<string, IRendererSurface> = new Map();
  /** The description each key's material was built from. */
  private readonly descriptions: Map<string, string> = new Map();
  /** The description each material was built from, which is the cache's key for it. */
  private readonly built: WeakMap<ISurfaceMaterial, string> = new WeakMap();
  /** Materials no surface names any more, disposed or cached once nothing draws them. */
  private readonly retired: Set<ISurfaceMaterial> = new Set();
  /** Compiled materials no surface names, by description, oldest first. */
  private readonly cache: Map<string, ISurfaceMaterial> = new Map();
  private isWireframe: boolean = false;

  /**
   * @param textures - Where the materials bind their textures.
   * @param uniforms - What their shaders read.
   * @param onReplaced - Told when a key's material changed, so whatever draws it can draw the new one.
   */
  public constructor(textures: RendererTextures, uniforms: RendererUniforms, onReplaced: (key: string) => void) {
    this.textures = textures;
    this.uniforms = uniforms;
    this.onReplaced = onReplaced;
  }

  /** Whether any material waits for nothing to draw it. */
  public get hasRetired(): boolean {
    return this.retired.size > 0;
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
      this.sources.set(key, surface);
      this.build(key, surface);
    }
  }

  public release(key: string): void {
    const previous: Maybe<ISurfaceMaterial> = this.materials.get(key);

    this.materials.delete(key);
    this.sources.delete(key);
    this.descriptions.delete(key);

    if (previous) {
      this.retired.add(previous);
    }

    this.onReplaced(key);
  }

  /**
   * @param isWireframe - Whether every surface draws as its triangles' edges.
   */
  public setWireframe(isWireframe: boolean): void {
    if (isWireframe !== this.isWireframe) {
      this.isWireframe = isWireframe;
      this.sources.forEach((surface: IRendererSurface, key: string) => this.build(key, surface));
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

    this.materials.clear();
    this.retired.clear();
    this.cache.clear();
    this.sources.clear();
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
      material = createSurfaceMaterial(surface, this.textures, this.uniforms);
      material.material.wireframe = this.isWireframe;
      this.built.set(material, description);
    }

    this.descriptions.set(key, description);
    this.materials.set(key, material);

    if (previous) {
      this.retired.add(previous);
    }

    this.onReplaced(key);
  }

  /** What a surface is built from: the surface itself and the wireframe setting, as one comparable string. */
  private toDescription(surface: IRendererSurface): string {
    return `${this.isWireframe ? "wireframe" : "solid"}:${JSON.stringify(surface)}`;
  }
}
