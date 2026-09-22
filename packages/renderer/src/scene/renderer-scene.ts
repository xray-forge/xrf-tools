import { Maybe } from "@xrf/types";
import { BufferGeometry, Material, Mesh, MeshBasicNodeMaterial, Scene } from "three/webgpu";

import { IRendererGeometry } from "#/contract/scene/renderer-geometry";
import { IRendererObject } from "#/contract/scene/renderer-object";
import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { TRendererTextureSource } from "#/contract/scene/renderer-texture-source";
import { createRendererBufferGeometry } from "#/scene/renderer-buffer-geometry";
import { RendererTextures } from "#/scene/renderer-textures";
import { createSurfaceMaterial, ISurfaceMaterial } from "#/scene/surface-material";
import { IDdsRefusal } from "#/texture/dds/dds-refusal";

/** What a slot draws with in the half of an object that does not draw it, or whose surface is missing. */
const HIDDEN: Material = new MeshBasicNodeMaterial({ visible: false });

/**  One object: a deferred and a forward mesh over one geometry, each drawing only its own slots. */
interface ISceneObject {
  object: IRendererObject;
  deferred: Mesh;
  forward: Mesh;
}

/**
 * Everything a consumer put, as the two scenes the frame draws.
 */
export class RendererScene {
  /** A mesh the object's matrix places directly, never recomposed from a position and rotation. */
  private static createMesh(): Mesh {
    const mesh: Mesh = new Mesh();

    mesh.matrixAutoUpdate = false;

    return mesh;
  }

  /** What fills the G-buffer. */
  public readonly deferred: Scene = new Scene();
  /** What is composited after it. */
  public readonly forward: Scene = new Scene();
  public readonly textures: RendererTextures;

  private readonly geometries: Map<string, BufferGeometry> = new Map();
  private readonly surfaces: Map<string, ISurfaceMaterial> = new Map();
  private readonly objects: Map<string, ISceneObject> = new Map();

  public constructor(onTextureRefused: (key: string, refusal: IDdsRefusal) => void) {
    this.textures = new RendererTextures(onTextureRefused);
  }

  public putTexture(key: string, source: TRendererTextureSource): void {
    this.textures.put(key, source);
  }

  public releaseTexture(key: string): void {
    this.textures.release(key);
  }

  public putGeometry(key: string, geometry: IRendererGeometry): void {
    this.geometries.get(key)?.dispose();
    this.geometries.set(key, createRendererBufferGeometry(geometry));
    this.rebuild((object: IRendererObject) => object.geometry === key);
  }

  public releaseGeometry(key: string): void {
    this.geometries.get(key)?.dispose();
    this.geometries.delete(key);
    this.rebuild((object: IRendererObject) => object.geometry === key);
  }

  public putSurface(key: string, surface: IRendererSurface): void {
    const previous: Maybe<ISurfaceMaterial> = this.surfaces.get(key);

    this.surfaces.set(key, createSurfaceMaterial(surface, this.textures));
    this.rebuild((object: IRendererObject) => object.surfaces.includes(key));
    previous?.dispose();
  }

  public releaseSurface(key: string): void {
    const previous: Maybe<ISurfaceMaterial> = this.surfaces.get(key);

    this.surfaces.delete(key);
    this.rebuild((object: IRendererObject) => object.surfaces.includes(key));
    previous?.dispose();
  }

  public putObject(key: string, object: IRendererObject): void {
    let entry: Maybe<ISceneObject> = this.objects.get(key);

    if (!entry) {
      entry = { deferred: RendererScene.createMesh(), forward: RendererScene.createMesh(), object };
      this.objects.set(key, entry);
    }

    entry.object = object;
    this.build(entry);
  }

  public releaseObject(key: string): void {
    const entry: Maybe<ISceneObject> = this.objects.get(key);

    if (entry) {
      entry.deferred.removeFromParent();
      entry.forward.removeFromParent();
      this.objects.delete(key);
    }
  }

  public dispose(): void {
    this.objects.forEach((entry: ISceneObject) => {
      entry.deferred.removeFromParent();
      entry.forward.removeFromParent();
    });
    this.objects.clear();
    this.surfaces.forEach((surface: ISurfaceMaterial) => surface.dispose());
    this.surfaces.clear();
    this.geometries.forEach((geometry: BufferGeometry) => geometry.dispose());
    this.geometries.clear();
    this.textures.dispose();
  }

  private rebuild(isAffected: (object: IRendererObject) => boolean): void {
    this.objects.forEach((entry: ISceneObject) => {
      if (isAffected(entry.object)) {
        this.build(entry);
      }
    });
  }

  /** Points both meshes at what the object names now, and shows each only where it has a slot to draw. */
  private build(entry: ISceneObject): void {
    const { object, deferred, forward } = entry;
    const geometry: Maybe<BufferGeometry> = this.geometries.get(object.geometry);

    if (!geometry) {
      deferred.removeFromParent();
      forward.removeFromParent();

      return;
    }

    const slots: number = Math.max(
      object.surfaces.length,
      ...geometry.groups.map((group) => (group.materialIndex ?? 0) + 1)
    );
    const surfaces: Array<Maybe<ISurfaceMaterial>> = Array.from({ length: slots }, (_, slot: number) =>
      this.surfaces.get(object.surfaces[slot])
    );

    for (const [mesh, scene, isDeferred] of [
      [deferred, this.deferred, true],
      [forward, this.forward, false],
    ] as const) {
      const materials: Array<Material> = surfaces.map((surface: Maybe<ISurfaceMaterial>) =>
        surface && surface.isDeferred === isDeferred ? surface.material : HIDDEN
      );

      mesh.geometry = geometry;
      mesh.material = materials;

      if (object.matrix) {
        mesh.matrix.fromArray(object.matrix);
      } else {
        mesh.matrix.identity();
      }

      mesh.matrixWorldNeedsUpdate = true;

      if (materials.some((material: Material) => material !== HIDDEN)) {
        scene.add(mesh);
      } else {
        mesh.removeFromParent();
      }
    }
  }
}
