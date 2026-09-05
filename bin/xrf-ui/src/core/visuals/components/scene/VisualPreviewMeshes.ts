import {
  BufferAttribute,
  BufferGeometry,
  Matrix4,
  Mesh,
  MeshStandardMaterial,
  Object3D,
  Skeleton,
  SkinnedMesh,
  Texture,
} from "three";

import { createSubmeshGeometry } from "@/core/visuals/components/scene/VisualPreviewScene.utils";
import { applyXrayBumpShading, IVisualBumpShading, IVisualBumpTextures } from "@/core/visuals/lib/visual-bump";
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
 * The three view toggles a material answers to.
 */
export interface IVisualMeshMaterialOptions {
  isWireframe: boolean;
  isCheckerVisible: boolean;
  /** Whether a submesh whose material bound a bump pair is shaded with it, or drawn flat for comparison. */
  isBumpVisible: boolean;
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
 * One drawn submesh: its mesh, its own material, the texture applied to it, and the bump shading patched onto it.
 */
interface IVisualSubmeshMesh {
  submesh: IVisualSubmeshViews;
  mesh: Mesh<BufferGeometry, MeshStandardMaterial>;
  texture: Nullable<Texture>;
  /** The switch of a material shading a bump pair, or null for a surface with no pair to shade. */
  bump: Nullable<IVisualBumpShading>;
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

      this.meshes.set(submesh.index, { submesh, mesh, texture: null, bump: null });
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
   * Draws one submesh with a texture, borrowing it.
   *
   * Applied per submesh rather than per model because a visual's children each declare their own reference and they
   * arrive one at a time, so a model shows its first texture without waiting for its last.
   *
   * Never freed here, on replacement or on disposal: one upload is drawn by every submesh naming that file, and a
   * scene placing several models would share it further still. Whoever loaded it frees it.
   *
   * @param submeshIndex - Index the submesh reports, which is what the backend resolved against.
   * @param texture - Uploaded texture to draw with, owned by whoever loaded it.
   */
  public applyTexture(submeshIndex: number, texture: Texture): void {
    const drawn: Optional<IVisualSubmeshMesh> = this.meshes.get(submeshIndex);

    // Not this model's to free, so a texture for a submesh it does not draw is simply left alone.
    if (!drawn) {
      return;
    }

    drawn.texture = texture;

    if (!this.materialOptions?.isCheckerVisible) {
      drawn.mesh.material.map = texture;
      drawn.mesh.material.needsUpdate = true;
    }
  }

  /**
   * Shades one submesh with its bump pair, borrowing both textures.
   *
   * The authored tangent basis goes onto the geometry here rather than at build time, because most submeshes bind no
   * pair and would carry two attributes nothing reads. The material is patched once; the view toggle then switches a
   * uniform, so comparing flat and bumped costs neither a recompile nor a re-upload.
   *
   * @param submeshIndex - Index the submesh reports, which is what the backend resolved against.
   * @param textures - The uploaded pair, owned by whoever loaded it.
   */
  public applyBump(submeshIndex: number, textures: IVisualBumpTextures): void {
    const drawn: Optional<IVisualSubmeshMesh> = this.meshes.get(submeshIndex);

    if (!drawn) {
      return;
    }

    drawn.mesh.geometry.setAttribute("xrayTangent", new BufferAttribute(drawn.submesh.tangents, 3));
    drawn.mesh.geometry.setAttribute("xrayBinormal", new BufferAttribute(drawn.submesh.binormals, 3));
    drawn.bump = applyXrayBumpShading(drawn.mesh.material, textures);
    drawn.bump.setEnabled(this.materialOptions?.isBumpVisible ?? true);
  }

  /**
   * Applies the view toggles that change how a surface is drawn.
   *
   * Retained as well as applied, because a texture or a bump pair arriving later has to know whether the checkerboard
   * is currently standing in for it and whether the bump is being compared away.
   *
   * @param options - Whether to draw as wireframe, whether the checkerboard covers every texture, and whether bumps
   *   are shaded.
   */
  public applyMaterialOptions(options: IVisualMeshMaterialOptions): void {
    this.materialOptions = options;

    for (const { mesh, texture, bump } of this.meshes.values()) {
      mesh.material.wireframe = options.isWireframe;
      mesh.material.map = options.isCheckerVisible ? this.checker : texture;
      mesh.material.needsUpdate = true;
      bump?.setEnabled(options.isBumpVisible);
    }
  }

  /**
   * Detaches every mesh and frees the geometry and materials it built.
   *
   * Textures are left alone: they are borrowed, and a level placing the same visual twice would free the second copy's
   * uploads out from under the first.
   */
  public dispose(): void {
    for (const { mesh } of this.meshes.values()) {
      this.parent.remove(mesh);
      mesh.geometry.dispose();
      mesh.material.dispose();
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
