import { BufferGeometry, Matrix4, Mesh, MeshStandardMaterial, Object3D, Skeleton, SkinnedMesh, Texture } from "three";

import { createSubmeshGeometry } from "@/core/visuals/components/scene/VisualPreviewScene.utils";
import {
  getVisualSubmeshLevel,
  IVisualModelViews,
  IVisualSubmeshLevel,
  IVisualSubmeshViews,
} from "@/core/visuals/lib/visual-views";
import { Nullable, Optional } from "@/lib/types/general";

const MESH_METALNESS: number = 0.05;
const MESH_ROUGHNESS: number = 0.75;

/**
 * The two view toggles a material answers to.
 */
export interface IVisualMeshMaterialOptions {
  isWireframe: boolean;
  isCheckerVisible: boolean;
}

/** What drawing a model's submeshes needs beyond the model itself. */
export interface IVisualPreviewMeshesOptions {
  meshColor: number;
  /** Stands in for every texture while the uv checkerboard is on. */
  checker: Texture;
  /** Skin the submeshes carrying links bind to, or null for a model with no skeleton. */
  skin: Nullable<Skeleton>;
  /** How far down each collapse chain to draw on arrival: 0 is full detail, 1 is the coarsest each submesh has. */
  detail: number;
}

/**
 * One drawn submesh: its mesh, its own material, and the texture applied to it.
 */
interface IVisualSubmeshMesh {
  mesh: Mesh<BufferGeometry, MeshStandardMaterial>;
  texture: Nullable<Texture>;
}

/**
 * Everything one model draws, and everything the renderer uploaded for it.
 */
export class VisualPreviewMeshes {
  /**
   * Builds and attaches a mesh for every submesh the model packed.
   *
   * @param model - Model views, whose submeshes are the geometry to draw.
   * @param parent - Scene node the meshes attach to.
   * @param options - Material colour, the checkerboard, the skin to bind to, and the detail to draw at.
   * @returns The drawn meshes, which may be none for a model that packed no geometry.
   */
  public static create(
    model: IVisualModelViews,
    parent: Object3D,
    options: IVisualPreviewMeshesOptions
  ): VisualPreviewMeshes {
    return new VisualPreviewMeshes(model, parent, options);
  }

  private readonly meshes: Map<number, IVisualSubmeshMesh> = new Map();
  private readonly submeshes: Array<IVisualSubmeshViews>;
  private readonly parent: Object3D;
  private readonly checker: Texture;

  /** The last material options applied, so a texture landing later knows whether the checker is covering it. */
  private materialOptions: Nullable<IVisualMeshMaterialOptions> = null;

  private constructor(model: IVisualModelViews, parent: Object3D, options: IVisualPreviewMeshesOptions) {
    this.parent = parent;
    this.checker = options.checker;
    this.submeshes = model.submeshes;

    for (const submesh of this.submeshes) {
      const mesh: Mesh<BufferGeometry, MeshStandardMaterial> = this.createMesh(submesh, options);

      this.meshes.set(submesh.index, { mesh, texture: null });
      this.parent.add(mesh);
    }
  }

  /**
   * Draws every mesh at a different point along its collapse chain.
   *
   * Only a draw range changes: all levels are already in the uploaded index buffer, so this touches no attribute and
   * costs no upload — which is what makes dragging the control smooth on a model carrying nine hundred levels.
   * Bounding spheres are left alone deliberately: they describe the same geometry, and refitting the camera on every
   * step would make comparing detail impossible.
   *
   * @param detail - How far down each chain to go: 0 is full detail, 1 is the coarsest each submesh has.
   */
  public setDetailLevel(detail: number): void {
    for (const submesh of this.submeshes) {
      const drawn: Optional<IVisualSubmeshMesh> = this.meshes.get(submesh.index);

      if (drawn) {
        const level: IVisualSubmeshLevel = getVisualSubmeshLevel(submesh, detail);

        drawn.mesh.geometry.setDrawRange(level.start, level.count);
      }
    }
  }

  /**
   * Puts a loaded texture on one submesh, taking ownership of it.
   *
   * Applied per submesh rather than per model because a visual's children each declare their own reference and they
   * arrive one at a time, so a model shows its first texture without waiting for its last.
   *
   * @param submeshIndex - Index the submesh reports, which is what the backend resolved against.
   * @param texture - Uploaded texture to take ownership of.
   */
  public applyTexture(submeshIndex: number, texture: Texture): void {
    const drawn: Optional<IVisualSubmeshMesh> = this.meshes.get(submeshIndex);

    if (!drawn) {
      texture.dispose();

      return;
    }

    // Idempotent because the caller is a react effect that re-runs whenever any texture lands, so it re-offers the
    // ones already applied. Without this the line below would dispose the texture still in use.
    if (drawn.texture === texture) {
      return;
    }

    drawn.texture?.dispose();
    drawn.texture = texture;

    if (!this.materialOptions?.isCheckerVisible) {
      drawn.mesh.material.map = texture;
      drawn.mesh.material.needsUpdate = true;
    }
  }

  /**
   * Applies the view toggles that change how a surface is drawn.
   *
   * Retained as well as applied, because a texture arriving later has to know whether the checkerboard is currently
   * standing in for it.
   *
   * @param options - Whether to draw as wireframe, and whether the checkerboard covers every texture.
   */
  public applyMaterialOptions(options: IVisualMeshMaterialOptions): void {
    this.materialOptions = options;

    for (const { mesh, texture } of this.meshes.values()) {
      mesh.material.wireframe = options.isWireframe;
      mesh.material.map = options.isCheckerVisible ? this.checker : texture;
      mesh.material.needsUpdate = true;
    }
  }

  /**
   * Detaches every mesh and frees what the renderer uploaded for it.
   *
   * Materials and textures are per submesh, so they are this model's to free: leaving them behind would leak one
   * upload per submesh every time the user opens another visual.
   */
  public dispose(): void {
    for (const { mesh, texture } of this.meshes.values()) {
      this.parent.remove(mesh);
      mesh.geometry.dispose();
      mesh.material.dispose();
      texture?.dispose();
    }

    this.meshes.clear();
  }

  /**
   * Builds one submesh's mesh, skinned when both it and the model can be.
   *
   * @param submesh - Views over the packed geometry.
   * @param options - Material colour, the skin to bind to, and the detail to draw at.
   * @returns The mesh, not yet attached.
   */
  private createMesh(
    submesh: IVisualSubmeshViews,
    options: IVisualPreviewMeshesOptions
  ): Mesh<BufferGeometry, MeshStandardMaterial> {
    const geometry: BufferGeometry = createSubmeshGeometry(submesh, options.detail);
    const material: MeshStandardMaterial = new MeshStandardMaterial({
      color: options.meshColor,
      metalness: MESH_METALNESS,
      roughness: MESH_ROUGHNESS,
    });

    // Skinned only when this submesh carries links and the model carries bones to bind them to.
    const isSkinned: boolean = Boolean(submesh.skinIndices && submesh.skinWeights && options.skin);

    if (!isSkinned || !options.skin) {
      const mesh: Mesh<BufferGeometry, MeshStandardMaterial> = new Mesh(geometry, material);

      mesh.name = submesh.label;

      return mesh;
    }

    const mesh: SkinnedMesh<BufferGeometry, MeshStandardMaterial> = new SkinnedMesh(geometry, material);

    mesh.name = submesh.label;
    // A skinned mesh is never frustum culled, because three.js measures it against its bind pose and a motion reaches
    // outside that.
    mesh.frustumCulled = false;
    // An identity bind matrix, because the vertices and the bone transforms are already in the same space: the backend
    // composed both into model space. Letting three.js take the mesh's own world matrix instead would work only as
    // long as nothing ever moved the mesh.
    mesh.bind(options.skin, new Matrix4());

    return mesh;
  }
}
