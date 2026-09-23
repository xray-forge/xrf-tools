import { Maybe, Nullable } from "@xrf/types";
import {
  BufferGeometry,
  Material,
  Matrix4,
  Mesh,
  MeshBasicNodeMaterial,
  Scene,
  Skeleton,
  SkinnedMesh,
} from "three/webgpu";

import { IRendererGeometry } from "#/contract/scene/renderer-geometry";
import { IRendererObject } from "#/contract/scene/renderer-object";
import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { TRendererTextureSource } from "#/contract/scene/renderer-texture-source";
import { createRendererBufferGeometry } from "#/scene/renderer-buffer-geometry";
import { RendererSkeletonEntry, RendererSkeletons } from "#/scene/renderer-skeletons";
import { RendererTextures } from "#/scene/renderer-textures";
import { createSurfaceMaterial, ISurfaceMaterial, ISurfaceShadingContext } from "#/scene/surface-material";
import { IDdsRefusal } from "#/texture/dds/dds-refusal";

/** What a slot draws with in the half of an object that does not draw it, or whose surface is missing. */
const HIDDEN: Material = new MeshBasicNodeMaterial({ visible: false });

/**  One object: a deferred and a forward mesh over one geometry, each drawing only its own slots. */
interface ISceneObject {
  object: IRendererObject;
  deferred: Mesh;
  forward: Mesh;
  /** The skeleton both meshes are bound to, or null for rigid meshes. */
  skeleton: Nullable<Skeleton>;
}

/**
 * Everything a consumer put, as the two scenes the frame draws.
 */
export class RendererScene {
  /**
   * A mesh the object's matrix places directly, never recomposed from a position and rotation.
   *
   * @param skeleton - What it is skinned to, or null for a rigid mesh.
   * @returns The mesh.
   */
  private static createMesh(skeleton: Nullable<Skeleton>): Mesh {
    const mesh: Mesh = skeleton ? new SkinnedMesh() : new Mesh();

    mesh.matrixAutoUpdate = false;

    if (skeleton) {
      // Measured against its bind pose, a skinned mesh would be culled by a motion reaching outside it.
      mesh.frustumCulled = false;
      // Identity: the vertices and the bone transforms are both in model space already.
      (mesh as SkinnedMesh).bind(skeleton, new Matrix4());
    }

    return mesh;
  }

  /** What fills the G-buffer. */
  public readonly deferred: Scene = new Scene();
  /** What is composited after it. */
  public readonly forward: Scene = new Scene();
  public readonly textures: RendererTextures;
  public readonly skeletons: RendererSkeletons;

  private readonly geometries: Map<string, BufferGeometry> = new Map();
  private readonly surfaces: Map<string, ISurfaceMaterial> = new Map();
  private readonly objects: Map<string, ISceneObject> = new Map();

  private readonly shading: ISurfaceShadingContext;

  private isWireframe: boolean = false;

  public constructor(shading: ISurfaceShadingContext, onTextureRefused: (key: string, refusal: IDdsRefusal) => void) {
    this.shading = shading;
    this.textures = new RendererTextures(onTextureRefused);
    this.skeletons = new RendererSkeletons((key: string) =>
      this.rebuild((object: IRendererObject) => object.skeleton === key)
    );
  }

  /**
   * @param isWireframe - Whether every surface draws as its triangles' edges.
   */
  public setWireframe(isWireframe: boolean): void {
    if (isWireframe === this.isWireframe) {
      return;
    }

    this.isWireframe = isWireframe;
    this.surfaces.forEach((surface: ISurfaceMaterial) => {
      surface.material.wireframe = isWireframe;
      surface.material.needsUpdate = true;
    });
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

    const material: ISurfaceMaterial = createSurfaceMaterial(surface, this.textures, this.shading);

    material.material.wireframe = this.isWireframe;
    this.surfaces.set(key, material);
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
      entry = {
        deferred: RendererScene.createMesh(null),
        forward: RendererScene.createMesh(null),
        object,
        skeleton: null,
      };
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
    this.skeletons.dispose();
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
    const { object } = entry;
    const geometry: Maybe<BufferGeometry> = this.geometries.get(object.geometry);

    if (!geometry) {
      entry.deferred.removeFromParent();
      entry.forward.removeFromParent();

      return;
    }

    // Skinned only when the object names a skeleton that exists and its geometry carries the links to bind with.
    const skeletonEntry: Maybe<RendererSkeletonEntry> = object.skeleton
      ? this.skeletons.get(object.skeleton)
      : undefined;
    const skeleton: Nullable<Skeleton> =
      skeletonEntry && geometry.hasAttribute("skinIndex") ? skeletonEntry.skeleton : null;

    if (skeleton !== entry.skeleton) {
      entry.deferred.removeFromParent();
      entry.forward.removeFromParent();
      entry.deferred = RendererScene.createMesh(skeleton);
      entry.forward = RendererScene.createMesh(skeleton);
      entry.skeleton = skeleton;
    }

    const { deferred, forward } = entry;

    geometry.setDrawRange(object.drawRange?.start ?? 0, object.drawRange?.count ?? Infinity);

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
