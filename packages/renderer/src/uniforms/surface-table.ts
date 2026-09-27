import { storage } from "three/tsl";
import { BufferAttribute, StorageBufferAttribute, StorageBufferNode } from "three/webgpu";

/** Words one row takes: the surface's numbers, then a layer for each slot it samples from an array. */
export const SURFACE_TABLE_WORDS: number = 16;

/** Where a row's layers start, one word a slot, in the order a material binds its slots. */
export const SURFACE_TABLE_LAYER_WORD: number = 8;

/** Layers a row holds. */
export const SURFACE_TABLE_LAYERS: number = SURFACE_TABLE_WORDS - SURFACE_TABLE_LAYER_WORD;

/** What a static draw's slot names for its row where its surface draws by a material of its own. */
export const SURFACE_NO_ROW: number = 0xffffffff;

/** Rows the table holds before it first grows. */
const INITIAL_ROWS: number = 1 << 10;

/** What one row says of a surface. */
export interface ISurfaceTableRow {
  tiling: number;
  detailScale: number;
  alphaReference: number;
  slice: number;
  color: readonly [number, number, number];
  /** Each slot's layer in the array its material samples it from, by the slot's place; zero for the rest. */
  layers: ReadonlyArray<number>;
}

/** The numbers and array layers of every surface a shared material draws, a row a surface, read by its slot's row. */
export class SurfaceTable {
  /** What bundles binding the table are told by when it grows, as a texture's key. */
  public readonly key: string = "surface-table";

  public rows: StorageBufferAttribute;
  /** The rows as one node every shared material reads, four words an element. */
  public readonly words: StorageBufferNode<"uvec4">;

  private readonly free: Array<number> = [];
  /** The rows written since the last upload, as one span. */
  private first: number = Infinity;
  private last: number = -1;
  private retired: Array<BufferAttribute> = [];
  private capacity: number = INITIAL_ROWS;
  private used: number = 0;
  private currentVersion: number = 0;

  public constructor() {
    this.rows = new StorageBufferAttribute(new Uint32Array(INITIAL_ROWS * SURFACE_TABLE_WORDS), 4);
    this.words = storage(this.rows, "uvec4", INITIAL_ROWS * (SURFACE_TABLE_WORDS / 4)).toReadOnly();
  }

  /** Bumped by every growth, which every bundle binding the table records again for. */
  public get version(): number {
    return this.currentVersion;
  }

  /** @returns A row free for a surface, the table grown where it holds no more. */
  public allocate(): number {
    if (this.free.length) {
      return this.free.pop() as number;
    }

    if (this.used === this.capacity) {
      this.grow(this.capacity * 2);
    }

    this.used += 1;

    return this.used - 1;
  }

  /**
   * @param row - A row no surface reads from now on.
   */
  public release(row: number): void {
    this.free.push(row);
  }

  /**
   * @param row - A surface's row.
   * @param values - What it says.
   */
  public write(row: number, values: ISurfaceTableRow): void {
    const words: Uint32Array = this.rows.array as Uint32Array;
    const numbers: Float32Array = new Float32Array(words.buffer, words.byteOffset, words.length);
    const at: number = row * SURFACE_TABLE_WORDS;

    numbers.set([values.tiling, values.detailScale, values.alphaReference, values.slice, ...values.color, 0], at);
    words.fill(0, at + SURFACE_TABLE_LAYER_WORD, at + SURFACE_TABLE_WORDS);
    words.set(values.layers.slice(0, SURFACE_TABLE_LAYERS), at + SURFACE_TABLE_LAYER_WORD);
    this.first = Math.min(this.first, row);
    this.last = Math.max(this.last, row);
  }

  /** Marks what changed since the last upload to go up with the next use of the buffer. */
  public flush(): void {
    if (this.last < this.first) {
      return;
    }

    // Joined with a span still queued rather than replacing it: three clears what it sent itself.
    const [queued] = this.rows.updateRanges;
    const start: number = Math.min(queued?.start ?? Infinity, this.first * SURFACE_TABLE_WORDS);
    const end: number = Math.max(queued ? queued.start + queued.count : 0, (this.last + 1) * SURFACE_TABLE_WORDS);

    this.rows.clearUpdateRanges();
    this.rows.addUpdateRange(start, end - start);
    this.rows.needsUpdate = true;
    this.first = Infinity;
    this.last = -1;
  }

  /** @returns The buffers growths replaced, to free once nothing binds them. */
  public takeRetired(): Array<BufferAttribute> {
    const retired: Array<BufferAttribute> = this.retired;

    this.retired = [];

    return retired;
  }

  private grow(capacity: number): void {
    const rows: Uint32Array = new Uint32Array(capacity * SURFACE_TABLE_WORDS);

    rows.set(this.rows.array as Uint32Array);
    this.retired.push(this.rows);
    this.rows = new StorageBufferAttribute(rows, 4);
    this.words.value = this.rows;
    this.capacity = capacity;
    this.currentVersion += 1;
    // A new buffer goes up whole.
    this.first = Infinity;
    this.last = -1;
  }
}
