import { Nullable } from "@xrf/types";

/** A square of the atlas, in texels from its top left corner. */
export interface ILightShadowTile {
  readonly x: number;
  readonly y: number;
  readonly size: number;
}

/**
 * Where the lights' shadow faces are drawn: one square texture cut into squares of powers of two, a quarter of a free
 * square taken at a time, and four free quarters joined back into their square.
 */
export class LightShadowAtlas {
  public readonly size: number;
  public readonly minimum: number;
  /** Every free square, by its key. */
  private readonly free: Map<string, ILightShadowTile> = new Map();

  /**
   * @param size - Texels the atlas is across, a power of two.
   * @param minimum - Texels the least square is across, a power of two no larger.
   */
  public constructor(size: number, minimum: number) {
    this.size = size;
    this.minimum = minimum;
    this.clear();
  }

  /** Texels the squares taken cover. */
  public get used(): number {
    let free: number = 0;

    this.free.forEach((tile: ILightShadowTile) => (free += tile.size * tile.size));

    return this.size * this.size - free;
  }

  /**
   * @param size - Texels the square is to be across: a power of two, no less than the least and no more than the atlas.
   * @returns The square, or null where no free square is as large.
   */
  public allocate(size: number): Nullable<ILightShadowTile> {
    let found: Nullable<ILightShadowTile> = null;

    // The least free square that holds it, so large squares stay whole for large faces.
    this.free.forEach((tile: ILightShadowTile) => {
      if (tile.size >= size && (!found || tile.size < found.size)) {
        found = tile;
      }
    });

    if (!found) {
      return null;
    }

    let tile: ILightShadowTile = found;

    this.free.delete(toKey(tile));

    while (tile.size > size) {
      const half: number = tile.size / 2;

      this.release({ size: half, x: tile.x + half, y: tile.y });
      this.release({ size: half, x: tile.x, y: tile.y + half });
      this.release({ size: half, x: tile.x + half, y: tile.y + half });
      tile = { size: half, x: tile.x, y: tile.y };
    }

    return tile;
  }

  /**
   * @param tile - A square taken, given back; joined with its three siblings where they are free too.
   */
  public release(tile: ILightShadowTile): void {
    let current: ILightShadowTile = tile;

    while (current.size < this.size) {
      const parent: number = current.size * 2;
      const x: number = current.x - (current.x % parent);
      const y: number = current.y - (current.y % parent);
      const siblings: Array<ILightShadowTile> = [
        { size: current.size, x, y },
        { size: current.size, x: x + current.size, y },
        { size: current.size, x, y: y + current.size },
        { size: current.size, x: x + current.size, y: y + current.size },
      ].filter((it) => it.x !== current.x || it.y !== current.y);

      if (!siblings.every((it) => this.free.has(toKey(it)))) {
        break;
      }

      siblings.forEach((it) => this.free.delete(toKey(it)));
      current = { size: parent, x, y };
    }

    this.free.set(toKey(current), current);
  }

  /** Gives every square back. */
  public clear(): void {
    this.free.clear();
    this.free.set(toKey({ size: this.size, x: 0, y: 0 }), { size: this.size, x: 0, y: 0 });
  }
}

function toKey(tile: ILightShadowTile): string {
  return `${tile.x}:${tile.y}:${tile.size}`;
}
