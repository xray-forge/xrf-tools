import { Color, InstancedMesh, Mesh, MeshStandardMaterial, Object3D } from "three";

import { createInstancedMesh, createInstanceGeometry } from "@/core/level/lib/level-instance-geometry";
import { ILoadedSector } from "@/core/level/lib/level-sector-set";
import { ISectorInstanceViews, ISectorSectionViews } from "@/core/level/lib/level-sector-views";
import { ILevelTexture, LevelTextureSet } from "@/core/level/lib/level-texture-set";
import { Maybe, Nullable } from "@/lib/types/general";

const SURFACE_METALNESS: number = 0.0;
const SURFACE_ROUGHNESS: number = 0.9;

/** Turns of the golden angle, which spreads consecutive shader ids rather than grouping them into near-identical hues. */
const HUE_STEP: number = 137.508;

/**
 * A stable colour per shader table entry, so one surface is the same colour in every sector of the level.
 *
 * @param shaderId - Entry of the level's shader table.
 * @returns A colour derived from the id rather than assigned in arrival order.
 */
export function getShaderColor(shaderId: number): Color {
  return new Color().setHSL(((shaderId * HUE_STEP) % 360) / 360, 0.45, 0.6);
}

/** How the surfaces of a level are drawn while a toggle is on. */
export interface ILevelSectorViewOptions {
  isWireframe: boolean;
  /** Draws each shader table entry in its own colour. */
  isSurfaceColored: boolean;
  /** Draws the surfaces with the textures the level dresses them in, or in flat colour for comparison. */
  isTextured: boolean;
}

export const DEFAULT_LEVEL_SECTOR_VIEW_OPTIONS: ILevelSectorViewOptions = {
  isSurfaceColored: false,
  isTextured: true,
  isWireframe: false,
};

/** One drawn sector: its mesh, and the material of each of its surfaces. */
interface IDrawnSector {
  sector: number;
  mesh: Mesh;
  materials: Array<MeshStandardMaterial>;
  /** The surface each material draws, parallel to `materials`: the sector's own sections, then its instanced meshes. */
  surfaces: Array<ISectorSurface>;
  /** One mesh per instanced group, each standing in every place the level puts it. */
  instanced: Array<InstancedMesh>;
}

/** What a material is dressed from, whichever way its geometry is drawn. */
type ISectorSurface = ISectorSectionViews | ISectorInstanceViews;

/** Lightmaps are baked light rather than a texture, so they multiply the surface rather than replacing it. */
const LIGHTMAP_INTENSITY: number = 1.0;

/**
 * Keeps one mesh per resident sector, in step with what the loader holds.
 */
export class LevelPreviewSectors {
  private readonly parent: Object3D;
  private readonly drawn: Map<number, IDrawnSector> = new Map();

  private options: ILevelSectorViewOptions = DEFAULT_LEVEL_SECTOR_VIEW_OPTIONS;

  /** Where a surface's textures come from, borrowed rather than owned: the loader disposes them. */
  private textures: Nullable<LevelTextureSet> = null;

  public constructor(parent: Object3D) {
    this.parent = parent;
  }

  /**
   * Takes the set a later sector dresses its surfaces from.
   *
   * @param textures - The open level's textures, owned by the loader.
   */
  public setTextures(textures: Nullable<LevelTextureSet>): void {
    this.textures = textures;
  }

  public get size(): number {
    return this.drawn.size;
  }

  /**
   * Adds a mesh for every sector newly resident and removes the meshes of sectors that are not.
   *
   * @param sectors - What the loader currently holds, keyed by sector.
   */
  public sync(sectors: ReadonlyMap<number, ILoadedSector>): void {
    for (const sector of Array.from(this.drawn.keys())) {
      if (!sectors.has(sector)) {
        this.remove(sector);
      }
    }

    for (const [sector, loaded] of sectors) {
      if (!this.drawn.has(sector)) {
        this.add(sector, loaded);
      }
    }
  }

  /**
   * Applies the view toggles to every drawn surface.
   *
   * @param options - What the toolbar has switched on.
   */
  public applyViewOptions(options: ILevelSectorViewOptions): void {
    this.options = options;

    for (const drawn of this.drawn.values()) {
      drawn.materials.forEach((material: MeshStandardMaterial, index: number) => {
        const surface: ISectorSurface | undefined = drawn.surfaces[index];

        material.wireframe = options.isWireframe;
        material.color = this.getSurfaceColor(surface?.shaderId ?? 0);

        this.dressMaterial(material, surface ?? null);

        material.needsUpdate = true;
      });
    }
  }

  /** Removes every mesh and disposes every material it made. */
  public dispose(): void {
    for (const sector of Array.from(this.drawn.keys())) {
      this.remove(sector);
    }
  }

  private add(sector: number, loaded: ILoadedSector): void {
    // A sector with no surface still gets one material, because a mesh whose groups have no material draws nothing
    // and says nothing about why.
    const surfaces: Array<ISectorSurface> = [...loaded.views.sections];
    const materials: Array<MeshStandardMaterial> = surfaces.length
      ? surfaces.map((surface: ISectorSurface) => this.createMaterial(surface.shaderId, surface))
      : [this.createMaterial(0, null)];
    const mesh: Mesh = new Mesh(loaded.geometry, materials);

    mesh.name = `sector-${sector}`;
    mesh.matrixAutoUpdate = false;

    const instanced: Array<InstancedMesh> = loaded.views.instances.map((group: ISectorInstanceViews) => {
      const material: MeshStandardMaterial = this.createMaterial(group.shaderId, group);

      materials.push(material);
      surfaces.push(group);

      return createInstancedMesh(group, createInstanceGeometry(group), material);
    });

    this.drawn.set(sector, { instanced, materials, mesh, sector, surfaces });
    this.parent.add(mesh);

    for (const tree of instanced) {
      this.parent.add(tree);
    }
  }

  private remove(sector: number): void {
    const drawn: Maybe<IDrawnSector> = this.drawn.get(sector);

    if (!drawn) {
      return;
    }

    this.parent.remove(drawn.mesh);

    // The sector's own geometry belongs to the loader, but an instanced mesh built its own here and has to free it.
    for (const tree of drawn.instanced) {
      this.parent.remove(tree);
      tree.geometry.dispose();
      tree.dispose();
    }

    for (const material of drawn.materials) {
      material.dispose();
    }

    this.drawn.delete(sector);
  }

  private createMaterial(shaderId: number, section: Nullable<ISectorSurface>): MeshStandardMaterial {
    const material: MeshStandardMaterial = new MeshStandardMaterial({
      color: this.getSurfaceColor(shaderId),
      metalness: SURFACE_METALNESS,
      roughness: SURFACE_ROUGHNESS,
      wireframe: this.options.isWireframe,
    });

    this.dressMaterial(material, section);

    return material;
  }

  /**
   * Puts a surface's own textures on its material.
   */
  private dressMaterial(material: MeshStandardMaterial, section: Nullable<ISectorSurface>): void {
    if (!section || !this.textures || !this.options.isTextured) {
      material.map = null;
      material.lightMap = null;

      return;
    }

    const base: Nullable<ILevelTexture> = section.textureName ? this.textures.get(section.textureName) : null;
    const lightmap: Nullable<ILevelTexture> = section.lightmaps[0] ? this.textures.get(section.lightmaps[0]) : null;

    material.map = base?.texture ?? null;
    material.lightMap = lightmap?.texture ?? null;
    material.lightMapIntensity = LIGHTMAP_INTENSITY;

    // A textured surface takes its colour from the texture, so the tint has to come off or every surface is dyed.
    if (material.map) {
      material.color = new Color(0xffffff);
    }
  }

  private getSurfaceColor(shaderId: number): Color {
    return this.options.isSurfaceColored ? getShaderColor(shaderId) : new Color(0xffffff);
  }
}
