import { Color, Mesh, MeshStandardMaterial, Object3D } from "three";

import { ILoadedSector } from "@/core/level/lib/level-sector-set";
import { ISectorSectionViews } from "@/core/level/lib/level-sector-views";
import { Maybe } from "@/lib/types/general";

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
}

export const DEFAULT_LEVEL_SECTOR_VIEW_OPTIONS: ILevelSectorViewOptions = {
  isSurfaceColored: true,
  isWireframe: false,
};

/** One drawn sector: its mesh, and the material of each of its surfaces. */
interface IDrawnSector {
  sector: number;
  mesh: Mesh;
  materials: Array<MeshStandardMaterial>;
  /** The shader table entry each material draws, parallel to `materials`. */
  shaderIds: Array<number>;
}

/**
 * Keeps one mesh per resident sector, in step with what the loader holds.
 */
export class LevelPreviewSectors {
  private readonly parent: Object3D;
  private readonly drawn: Map<number, IDrawnSector> = new Map();

  private options: ILevelSectorViewOptions = DEFAULT_LEVEL_SECTOR_VIEW_OPTIONS;

  public constructor(parent: Object3D) {
    this.parent = parent;
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
        material.wireframe = options.isWireframe;
        material.color = this.getSurfaceColor(drawn.shaderIds[index] ?? 0);
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
    const shaderIds: Array<number> = loaded.views.sections.length
      ? loaded.views.sections.map((section: ISectorSectionViews) => section.shaderId)
      : [0];
    const materials: Array<MeshStandardMaterial> = shaderIds.map((shaderId: number) => this.createMaterial(shaderId));
    const mesh: Mesh = new Mesh(loaded.geometry, materials);

    mesh.name = `sector-${sector}`;
    mesh.matrixAutoUpdate = false;

    this.drawn.set(sector, { materials, mesh, sector, shaderIds });
    this.parent.add(mesh);
  }

  private remove(sector: number): void {
    const drawn: Maybe<IDrawnSector> = this.drawn.get(sector);

    if (!drawn) {
      return;
    }

    this.parent.remove(drawn.mesh);

    for (const material of drawn.materials) {
      material.dispose();
    }

    this.drawn.delete(sector);
  }

  private createMaterial(shaderId: number): MeshStandardMaterial {
    return new MeshStandardMaterial({
      color: this.getSurfaceColor(shaderId),
      metalness: SURFACE_METALNESS,
      roughness: SURFACE_ROUGHNESS,
      wireframe: this.options.isWireframe,
    });
  }

  private getSurfaceColor(shaderId: number): Color {
    return this.options.isSurfaceColored ? getShaderColor(shaderId) : new Color(0xffffff);
  }
}
