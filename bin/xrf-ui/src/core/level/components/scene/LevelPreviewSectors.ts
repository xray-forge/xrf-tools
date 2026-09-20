import { InstancedMesh, Mesh, Object3D } from "three";

import { createInstancedMesh } from "@/core/level/lib/level-instance-geometry";
import { LevelMaterialSet } from "@/core/level/lib/level-material-set";
import { createGeometry } from "@/core/level/lib/level-sector-geometry";
import { ILoadedSector } from "@/core/level/lib/level-sector-set";
import { ISectorInstanceViews, ISectorSectionViews } from "@/core/level/lib/level-sector-views";
import {
  DEFAULT_LEVEL_SURFACE_OPTIONS,
  ILevelSurface,
  ILevelSurfaceOptions,
} from "@/core/level/lib/level-surface-material";
import { ILevelTextureLookup } from "@/core/level/lib/level-texture-set";
import { Maybe, Nullable } from "@/lib/types/general";

/** One drawn sector: the mesh of everything baked in place, the meshes it stands, and the surfaces drawing them. */
interface IDrawnSector {
  sector: number;
  /** Absent for a sector the level bakes nothing of, which is every one whose drawables it all places. */
  mesh: Nullable<Mesh>;
  /** One per mesh the sector stands in many places, each already holding every place. */
  instanced: Array<InstancedMesh>;
  /** The sector's own sections first, then its instanced meshes, in the order their materials were claimed. */
  surfaces: Array<ILevelSurface>;
}

/**
 * Keeps one mesh per resident sector, in step with what the loader holds.
 */
export class LevelPreviewSectors {
  private readonly parent: Object3D;
  private readonly drawn: Map<number, IDrawnSector> = new Map();
  private readonly materials: LevelMaterialSet = new LevelMaterialSet();

  public constructor(parent: Object3D) {
    this.parent = parent;
  }

  public get size(): number {
    return this.drawn.size;
  }

  /**
   * @returns Distinct materials held, which is what the sectors on screen really cost.
   */
  public get materialCount(): number {
    return this.materials.size;
  }

  /**
   * Takes the set a later surface is dressed from.
   *
   * @param textures - The open level's textures, owned by the loader.
   */
  public setTextures(textures: Nullable<ILevelTextureLookup>): void {
    this.materials.setTextures(textures);
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

    // After both, so a surface a departing sector named and an arriving one still names is never disposed and then
    // built again between the two.
    this.materials.retain(this.listDrawnSurfaces());
  }

  /**
   * Applies the view toggles to every surface on screen.
   *
   * @param options - What the toolbar has switched on.
   */
  public applyViewOptions(options: ILevelSurfaceOptions = DEFAULT_LEVEL_SURFACE_OPTIONS): void {
    this.materials.applyViewOptions(options);
  }

  /** Removes every mesh and disposes everything it made. */
  public dispose(): void {
    for (const sector of Array.from(this.drawn.keys())) {
      this.remove(sector);
    }

    this.materials.dispose();
  }

  private add(sector: number, loaded: ILoadedSector): void {
    // The sector's own mesh is one geometry for every section, so whether it carries baked vertex colour is the
    // sector's answer rather than each section's.
    const hasVertexColors: boolean = loaded.views.geometry.colors !== null;
    const surfaces: Array<ILevelSurface> = loaded.views.sections.map((section: ISectorSectionViews) => ({
      hasVertexColors,
      render: section.render,
      surface: section.surface,
    }));
    // Only where the level bakes something in place. A sector whose drawables it all places has an empty index array,
    // and a mesh drawing none of it would be a draw call to say nothing.
    const mesh: Nullable<Mesh> = surfaces.length ? this.createMesh(sector, loaded, surfaces) : null;

    const instanced: Array<InstancedMesh> = loaded.views.instances.map((group: ISectorInstanceViews) => {
      const surface: ILevelSurface = {
        hasVertexColors: group.geometry.colors !== null,
        render: group.render,
        surface: group.surface,
      };

      surfaces.push(surface);

      return createInstancedMesh(group, createGeometry(group.geometry), this.materials.claim(surface));
    });

    this.drawn.set(sector, { instanced, mesh, sector, surfaces });

    for (const drawn of mesh ? [mesh, ...instanced] : instanced) {
      this.parent.add(drawn);
    }
  }

  /** The one mesh everything the level bakes in place is drawn from, a group to each of its surfaces. */
  private createMesh(sector: number, loaded: ILoadedSector, surfaces: Array<ILevelSurface>): Mesh {
    const mesh: Mesh = new Mesh(
      loaded.geometry,
      surfaces.map((surface: ILevelSurface) => this.materials.claim(surface))
    );

    mesh.name = `sector-${sector}`;
    mesh.matrixAutoUpdate = false;

    return mesh;
  }

  private remove(sector: number): void {
    const drawn: Maybe<IDrawnSector> = this.drawn.get(sector);

    if (!drawn) {
      return;
    }

    if (drawn.mesh) {
      this.parent.remove(drawn.mesh);
    }

    // The sector's own geometry belongs to the loader, but an instanced mesh took its own here and has to free it.
    // Materials are not freed here: another sector may still draw the same surface, so `retain` decides.
    for (const tree of drawn.instanced) {
      this.parent.remove(tree);
      tree.geometry.dispose();
      tree.dispose();
    }

    this.drawn.delete(sector);
  }

  /** Every surface the sectors on screen name, which is what is worth keeping a material for. */
  private *listDrawnSurfaces(): Iterable<ILevelSurface> {
    for (const drawn of this.drawn.values()) {
      yield* drawn.surfaces;
    }
  }
}
