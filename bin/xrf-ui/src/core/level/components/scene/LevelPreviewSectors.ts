import { InstancedMesh, Mesh, MeshStandardMaterial, Object3D } from "three";

import { SectorSurface } from "@/core/ipc/types/xrf-visual";
import { createInstancedMesh } from "@/core/level/lib/level-instance-geometry";
import { createGeometry } from "@/core/level/lib/level-sector-geometry";
import { ILoadedSector } from "@/core/level/lib/level-sector-set";
import { ISectorInstanceViews, ISectorSectionViews } from "@/core/level/lib/level-sector-views";
import {
  createSurfaceMaterial,
  DEFAULT_LEVEL_SURFACE_OPTIONS,
  dressSurfaceMaterial,
  ILevelSurfaceOptions,
} from "@/core/level/lib/level-surface-material";
import { LevelTextureSet } from "@/core/level/lib/level-texture-set";
import { Maybe, Nullable } from "@/lib/types/general";

/** One drawn surface: the material it uses, and what the shader table says dresses it. */
interface IDrawnSurface {
  material: MeshStandardMaterial;
  surface: Nullable<SectorSurface>;
}

/** One drawn sector: the mesh of everything baked in place, the meshes it stands, and their materials. */
interface IDrawnSector {
  sector: number;
  mesh: Mesh;
  /** One per mesh the sector stands in many places, each already holding every place. */
  instanced: Array<InstancedMesh>;
  /** The sector's own sections first, then its instanced meshes, in the order their materials were made. */
  surfaces: Array<IDrawnSurface>;
}

/**
 * Keeps one mesh per resident sector, in step with what the loader holds.
 */
export class LevelPreviewSectors {
  private readonly parent: Object3D;
  private readonly drawn: Map<number, IDrawnSector> = new Map();

  private options: ILevelSurfaceOptions = DEFAULT_LEVEL_SURFACE_OPTIONS;

  /** Where a surface's textures come from, borrowed rather than owned: the loader disposes them. */
  private textures: Nullable<LevelTextureSet> = null;

  public constructor(parent: Object3D) {
    this.parent = parent;
  }

  public get size(): number {
    return this.drawn.size;
  }

  /**
   * Takes the set a later sector dresses its surfaces from.
   *
   * @param textures - The open level's textures, owned by the loader.
   */
  public setTextures(textures: Nullable<LevelTextureSet>): void {
    this.textures = textures;
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
  public applyViewOptions(options: ILevelSurfaceOptions): void {
    this.options = options;

    for (const drawn of this.drawn.values()) {
      for (const { material, surface } of drawn.surfaces) {
        dressSurfaceMaterial(material, surface, this.textures, options);
      }
    }
  }

  /** Removes every mesh and disposes everything it made. */
  public dispose(): void {
    for (const sector of Array.from(this.drawn.keys())) {
      this.remove(sector);
    }
  }

  private add(sector: number, loaded: ILoadedSector): void {
    // A sector with no surface still gets one material, because a mesh whose groups have no material draws nothing
    // and says nothing about why.
    const sections: Array<Nullable<SectorSurface>> = loaded.views.sections.length
      ? loaded.views.sections.map((section: ISectorSectionViews) => section.surface)
      : [null];
    const surfaces: Array<IDrawnSurface> = sections.map((surface) => this.createSurface(surface));
    const mesh: Mesh = new Mesh(
      loaded.geometry,
      surfaces.map(({ material }) => material)
    );

    mesh.name = `sector-${sector}`;
    mesh.matrixAutoUpdate = false;

    const instanced: Array<InstancedMesh> = loaded.views.instances.map((group: ISectorInstanceViews) => {
      const drawn: IDrawnSurface = this.createSurface(group.surface);

      surfaces.push(drawn);

      return createInstancedMesh(group, createGeometry(group.geometry), drawn.material);
    });

    this.drawn.set(sector, { instanced, mesh, sector, surfaces });
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

    // The sector's own geometry belongs to the loader, but an instanced mesh took its own here and has to free it.
    for (const tree of drawn.instanced) {
      this.parent.remove(tree);
      tree.geometry.dispose();
      tree.dispose();
    }

    for (const { material } of drawn.surfaces) {
      material.dispose();
    }

    this.drawn.delete(sector);
  }

  private createSurface(surface: Nullable<SectorSurface>): IDrawnSurface {
    return { material: createSurfaceMaterial(surface, this.textures, this.options), surface };
  }
}
