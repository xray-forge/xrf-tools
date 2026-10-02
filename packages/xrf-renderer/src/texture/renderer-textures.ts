import { IDdsRefusal } from "@xrf/dds";
import { Maybe, Nullable } from "@xrf/types";
import { nodeObject } from "three/tsl";
import { CubeTexture, Node, Texture, TextureNode, WebGPURenderer } from "three/webgpu";

import { IRendererTextureFetch } from "#/contract/scene/renderer-texture-fetch";
import { ERendererTextureEncoding, TRendererTextureSource } from "#/contract/scene/renderer-texture-source";
import { isTextureOnGpu } from "#/internals/texture-residency";
import { fetchRendererTexture } from "#/texture/fetch-renderer-texture";
import {
  createRendererImageTexture,
  createRendererRawTexture,
  createRendererTexture,
} from "#/texture/renderer-texture";
import { IRendererTextureLoad } from "#/texture/renderer-texture-load";
import { IRendererTextureUpload } from "#/texture/renderer-texture-upload";
import { SlotTextureNode } from "#/texture/slot-texture-node";
import { hasTextureData, listTextureData, releaseTextureData } from "#/texture/texture-data";
import { ITextureTarget } from "#/texture/texture-target";

/** A put the renderer fetches itself, which it can fetch again. */
type TFetchedSource = Extract<TRendererTextureSource, { encoding: ERendererTextureEncoding.FETCH }>;

/** Something drawing a key's texture: what it draws while the key holds nothing, and whether it keeps it up. */
interface ITextureBinding {
  placeholder: Texture;
  /** Whether it draws the key plainly, so no eviction lets the texture go while it is bound: a sampler of `bind`. */
  isHolding: boolean;
}

/** One key's texture and everything drawing it. */
interface ITextureEntry {
  /** The texture last put, which may not be on the GPU yet. */
  texture: Nullable<Texture>;
  /** What the targets draw: always a texture already on the GPU, or nothing. */
  drawn: Nullable<Texture>;
  samplers: Map<ITextureTarget, ITextureBinding>;
  /** Bumped by every put and release, so a picture decoding or a file fetched late can tell it was superseded. */
  version: number;
  /** Whether a picture is still decoding for it, or its file still fetching. */
  isLoading: boolean;
  /** Aborts the fetch in flight for it, or null for none. */
  fetching: Nullable<AbortController>;
  /**
   * Where its latest put is fetched from, or null for one handed over as bytes: a fetched texture lets its bytes go once
   * up, and is fetched again where it has to go up again and no array's layer holds a copy of it.
   */
  refetch: Nullable<TFetchedSource>;
  /** Whether what it draws was let go on the GPU, an array's layer holding it: held again, it goes up again. */
  isEvicted: boolean;
  /** Holds of whatever draws its own texture; no eviction lets the texture go while any stays. */
  holds: number;
}

/**
 * The textures a consumer put, by key, bound into whatever samples them without recompiling anything.
 * A put texture goes up to the GPU in the frame loop, a few milliseconds a frame, and its samplers switch to it once it
 * is there: three uploads a texture the first time a binding needs it, which put a level's textures into one frame.
 * A texture the renderer fetched lets its bytes go once it is up, three never reading them again; one evicted and held
 * again is copied back on the GPU from the layer an array holds of it, and fetched anew only where none does. Asking
 * whether a key is up changes nothing: only a hold brings an evicted key back.
 */
export class RendererTextures {
  private readonly entries: Map<string, ITextureEntry> = new Map();
  /** Keys whose texture is not on the GPU yet, in the order they were put. */
  private readonly queued: Set<string> = new Set();
  /** Keys whose texture is still decoding or fetching, so it is not even queued yet. */
  private readonly loading: Set<string> = new Set();
  private readonly onRefused: (key: string, refusal: IDdsRefusal) => void;
  private readonly onRebound: (key: string) => void;
  private readonly onFetched: (key: string, fetch: IRendererTextureFetch) => void;
  private readonly restore: (renderer: WebGPURenderer, key: string, texture: Texture) => boolean;

  /**
   * @param onRefused - Told a file could not be uploaded as stored.
   * @param onRebound - Told a key's samplers were pointed at another texture, which a recorded bundle drawing them
   *   does not see until it is recorded again.
   * @param onFetched - Told what a fetched texture came to, once its fetch settles for the put that asked for it.
   * @param restore - Fills an evicted key's texture, its bytes gone, again on the GPU from the layer an array holds of
   *   it; false where none does, for it to be fetched again.
   */
  public constructor(
    onRefused: (key: string, refusal: IDdsRefusal) => void,
    onRebound: (key: string) => void,
    onFetched: (key: string, fetch: IRendererTextureFetch) => void = () => {},
    restore: (renderer: WebGPURenderer, key: string, texture: Texture) => boolean = () => false
  ) {
    this.onRefused = onRefused;
    this.onRebound = onRebound;
    this.onFetched = onFetched;
    this.restore = restore;
  }

  /**
   * @param key - What the texture is put under.
   * @param source - Its bytes.
   */
  public put(key: string, source: TRendererTextureSource): void {
    const entry: ITextureEntry = this.getEntry(key);
    const version: number = ++entry.version;

    // Whatever an earlier put still decodes or fetches is superseded, and would otherwise leave the key loading.
    this.stopLoading(key, entry);
    entry.refetch = source.encoding === ERendererTextureEncoding.FETCH ? source : null;

    if (source.encoding === ERendererTextureEncoding.FETCH) {
      this.fetch(key, entry, source, version);

      return;
    }

    if (source.encoding === ERendererTextureEncoding.DDS) {
      const upload: IRendererTextureUpload = createRendererTexture(source.bytes);

      this.assign(key, entry, upload.texture);

      if (upload.refusal) {
        this.onRefused(key, upload.refusal);
      }

      return;
    }

    if (source.encoding === ERendererTextureEncoding.RGBA) {
      this.assign(key, entry, createRendererRawTexture(source.bytes, source.width, source.height, source.isNearest));

      return;
    }

    this.startLoading(key, entry);

    createRendererImageTexture(source.bytes, source.type)
      .then((texture: Texture) => {
        if (entry.version === version && this.entries.get(key) === entry) {
          this.stopLoading(key, entry);
          this.assign(key, entry, texture);
        } else {
          texture.dispose();
        }
      })
      .catch(() => {
        if (entry.version === version && this.entries.get(key) === entry) {
          this.stopLoading(key, entry);
          this.assign(key, entry, null);
        }
      });
  }

  /**
   * @param key - What to let go of; whatever samples it goes back to its placeholder.
   */
  public release(key: string): void {
    const entry: Maybe<ITextureEntry> = this.entries.get(key);

    if (!entry) {
      return;
    }

    entry.version += 1;
    this.stopLoading(key, entry);
    this.assign(key, entry, null);
    this.prune(key, entry);
  }

  /**
   * A sampler of whatever the key holds, now and after every later put.
   *
   * @param key - The texture's key, or nothing for a slot the surface leaves empty.
   * @param placeholder - What it samples while the key holds nothing on the GPU.
   * @param coordinates - Where it samples.
   * @returns The sampler.
   */
  public bind(key: Maybe<string>, placeholder: Texture, coordinates: Node): TextureNode {
    // Its own binding: samplers built holding one placeholder would otherwise share one and all sample the first.
    const sampler: TextureNode = nodeObject(new SlotTextureNode(placeholder, coordinates)) as unknown as TextureNode;

    if (!key) {
      return sampler;
    }

    // Drawn plainly by whatever builds with it, so its key's own texture stays up however an array holds it.
    this.attach(key, placeholder, sampler, true);

    return sampler;
  }

  /**
   * Points a target at whatever the key holds, now and after every later put. A target holds nothing up: what draws it
   * holds its keys, and an evicted key's targets draw their placeholders.
   *
   * @param key - The texture's key.
   * @param placeholder - What it draws while the key holds nothing on the GPU.
   * @param target - What draws it.
   */
  public target(key: string, placeholder: Texture, target: ITextureTarget): void {
    this.attach(key, placeholder, target, false);
  }

  /**
   * @param key - The key a sampler or target was bound to.
   * @param sampler - What drew it, whose material is going away.
   */
  public unbind(key: Maybe<string>, sampler: ITextureTarget): void {
    const entry: Maybe<ITextureEntry> = key ? this.entries.get(key) : undefined;

    if (key && entry) {
      if (entry.samplers.get(sampler)?.isHolding) {
        entry.holds -= 1;
      }

      entry.samplers.delete(sampler);
      this.prune(key, entry);
    }
  }

  /**
   * Keeps keys' own textures on the GPU for something about to draw them or drawing them, whatever an array holds: the
   * one way an evicted key goes up again, with the uploads. Each hold is let go of once, by `letGo`.
   *
   * @param keys - The keys drawn, a key held as often as it is named.
   */
  public hold(keys: Iterable<string>): void {
    for (const key of keys) {
      const entry: ITextureEntry = this.getEntry(key);

      entry.holds += 1;

      if (entry.isEvicted) {
        this.queued.add(key);
      }
    }
  }

  /**
   * @param keys - Keys held before, which what held them draws no more.
   */
  public letGo(keys: Iterable<string>): void {
    for (const key of keys) {
      const entry: Maybe<ITextureEntry> = this.entries.get(key);

      if (entry) {
        entry.holds -= 1;
        this.prune(key, entry);
      }
    }
  }

  /**
   * @param key - A texture's key.
   * @returns Whether what the key holds is on the GPU, so drawing it plainly stalls nothing; true for a key holding
   *   nothing, whose samplers draw their placeholder. An evicted key is not, and only a hold brings it back.
   */
  public isUploaded(key: string): boolean {
    const entry: Maybe<ITextureEntry> = this.entries.get(key);

    return !entry || this.isResident(entry);
  }

  /**
   * @param key - A texture's key.
   * @returns What it holds on the GPU and draws, or null for one holding nothing there yet; an evicted key holds
   *   nothing there, and only a hold brings it back.
   */
  public getUploaded(key: string): Nullable<Texture> {
    const entry: Maybe<ITextureEntry> = this.entries.get(key);

    return entry && this.isResident(entry) ? entry.drawn : null;
  }

  /**
   * Lets a key's texture go on the GPU, an array's layer holding a copy of it. Its targets draw their placeholders, so
   * nothing built or drawn after has three upload it again; whatever holds it has it uploaded within the budget.
   *
   * @param key - A texture's key.
   * @returns The texture let go, or null for one not let go: holding nothing on the GPU, its latest put not up yet, or
   *   held by something drawing it.
   */
  public evict(key: string): Nullable<Texture> {
    const entry: Maybe<ITextureEntry> = this.entries.get(key);

    if (!entry?.drawn || entry.holds > 0 || entry.isEvicted || entry.isLoading || entry.drawn !== entry.texture) {
      return null;
    }

    entry.isEvicted = true;
    entry.samplers.forEach(({ placeholder }: ITextureBinding, target: ITextureTarget) => (target.value = placeholder));
    entry.drawn.dispose();

    return entry.drawn;
  }

  /**
   * @param key - A texture's key.
   * @returns Whether its texture was let go on the GPU and has not gone up again since.
   */
  public isEvicted(key: string): boolean {
    return this.entries.get(key)?.isEvicted === true;
  }

  /**
   * @returns The byte arrays the textures hold on the CPU: those not up yet, and those up that cannot be fetched again.
   */
  public listHeldData(): Array<ArrayBufferView> {
    const held: Array<ArrayBufferView> = [];

    this.entries.forEach(({ texture, drawn }: ITextureEntry) => {
      held.push(...(texture ? listTextureData(texture) : []));

      if (drawn && drawn !== texture) {
        held.push(...listTextureData(drawn));
      }
    });

    return held;
  }

  /** Whether any texture waits to go up, one still decoding or fetching included. */
  public get hasQueued(): boolean {
    return this.queued.size > 0 || this.loading.size > 0;
  }

  /**
   * Uploads queued textures, oldest first, until either budget is spent. The bytes bound what the GPU process copies a
   * frame, which the worker's clock does not: it only hands the bytes over, and the GPU process copies them again.
   *
   * @param renderer - The renderer uploading.
   * @param milliseconds - Worker time to spend; at least one texture goes up whatever it costs.
   * @param bytes - Bytes to send; a copy back from an array's layer sends none.
   */
  public upload(renderer: WebGPURenderer, milliseconds: number, bytes: number = Infinity): void {
    const started: number = performance.now();
    let sent: number = 0;

    for (const key of this.queued) {
      this.queued.delete(key);

      const entry: Maybe<ITextureEntry> = this.entries.get(key);

      // One loading is queued again as its bytes come.
      if (!entry?.texture || entry.isLoading || (entry.drawn === entry.texture && !entry.isEvicted)) {
        continue;
      }

      if (hasTextureData(entry.texture)) {
        sent += listTextureData(entry.texture).reduce(
          (total: number, data: ArrayBufferView) => total + data.byteLength,
          0
        );
        renderer.initTexture(entry.texture);

        // Sent: three reads the bytes of a texture it holds never again, so only one it can fetch again lets them go.
        if (entry.refetch && isTextureOnGpu(renderer, entry.texture)) {
          releaseTextureData(entry.texture);
        }
      } else if (entry.holds === 0 || !this.restore(renderer, key, entry.texture)) {
        // Evicted with its bytes gone and no layer holding it: fetched again while something still holds it, and
        // queued once it comes.
        if (entry.refetch && entry.holds > 0) {
          this.fetch(key, entry, entry.refetch, ++entry.version);
        }

        continue;
      }

      this.draw(key, entry, entry.texture);

      if (sent >= bytes || performance.now() - started >= milliseconds) {
        return;
      }
    }
  }

  public dispose(): void {
    // An evicted one too: three keeps nothing of a texture once it went, so a second dispose frees nothing twice.
    this.entries.forEach(({ drawn, texture, fetching }: ITextureEntry) => {
      fetching?.abort();
      texture?.dispose();

      if (drawn !== texture) {
        drawn?.dispose();
      }
    });
    this.entries.clear();
    this.queued.clear();
    this.loading.clear();
  }

  private getEntry(key: string): ITextureEntry {
    let entry: Maybe<ITextureEntry> = this.entries.get(key);

    if (!entry) {
      entry = {
        drawn: null,
        fetching: null,
        holds: 0,
        isEvicted: false,
        isLoading: false,
        refetch: null,
        samplers: new Map(),
        texture: null,
        version: 0,
      };
      this.entries.set(key, entry);
    }

    return entry;
  }

  /**
   * Takes a key's new texture. Its samplers keep drawing what they drew until it is uploaded, so a replacement never
   * flashes the placeholder; nothing at all takes them back to the placeholder at once.
   */
  private assign(key: string, entry: ITextureEntry, texture: Nullable<Texture>): void {
    // One put over another that never went up: the first was never drawn, so nothing is left sampling it.
    if (entry.texture && entry.texture !== texture && entry.texture !== entry.drawn) {
      entry.texture.dispose();
    }

    entry.texture = texture;
    this.queued.delete(key);

    if (texture) {
      this.queued.add(key);
    } else {
      this.draw(key, entry, null);
    }
  }

  /** Whether what a key holds is on the GPU and drawn: not evicted, loading, or put again and not up yet. */
  private isResident(entry: ITextureEntry): boolean {
    return !entry.isEvicted && !entry.isLoading && entry.drawn === entry.texture;
  }

  /**
   * Points every sampler of an entry at what it draws now, letting go of what it drew before. A texture uploaded again
   * after its eviction is told of too, since what waited for it can draw now.
   */
  private draw(key: string, entry: ITextureEntry, texture: Nullable<Texture>): void {
    const previous: Nullable<Texture> = entry.drawn;
    const wasEvicted: boolean = entry.isEvicted;

    entry.drawn = texture;
    entry.isEvicted = false;
    entry.samplers.forEach(
      ({ placeholder }: ITextureBinding, sampler: ITextureTarget) => (sampler.value = toDrawn(texture, placeholder))
    );

    if (entry.samplers.size && (previous !== texture || wasEvicted)) {
      this.onRebound(key);
    }

    // An evicted one too, which frees nothing twice.
    if (previous && previous !== texture) {
      previous.dispose();
    }
  }

  /** Fetches a key's file for the entry's version given, taking what it comes to unless that is superseded since. */
  private fetch(key: string, entry: ITextureEntry, source: TFetchedSource, version: number): void {
    const fetching: AbortController = new AbortController();

    entry.fetching = fetching;
    this.startLoading(key, entry);

    void fetchRendererTexture(source.file, source.picture, fetching.signal).then((load: IRendererTextureLoad) => {
      if (entry.version !== version || this.entries.get(key) !== entry) {
        load.texture?.dispose();

        return;
      }

      this.stopLoading(key, entry);
      this.assign(key, entry, load.texture);
      this.onFetched(key, load.fetch);
    });
  }

  /** Marks a key as decoding or fetching: not uploaded, and waited for by a settle. */
  private startLoading(key: string, entry: ITextureEntry): void {
    entry.isLoading = true;
    this.loading.add(key);
  }

  /** Ends whatever a key was decoding or fetching, aborting a fetch still in flight. */
  private stopLoading(key: string, entry: ITextureEntry): void {
    entry.fetching?.abort();
    entry.fetching = null;
    entry.isLoading = false;
    this.loading.delete(key);
  }

  /** Binds a target to a key, drawing what the key holds on the GPU now, which for an evicted key is nothing. */
  private attach(key: string, placeholder: Texture, target: ITextureTarget, isHolding: boolean): void {
    const entry: ITextureEntry = this.getEntry(key);

    target.value = entry.isEvicted ? placeholder : toDrawn(entry.drawn, placeholder);
    entry.samplers.set(target, { isHolding, placeholder });

    if (isHolding) {
      this.hold([key]);
    }
  }

  /** Forgets a key nothing holds and nothing samples. */
  private prune(key: string, entry: ITextureEntry): void {
    if (!entry.texture && !entry.drawn && entry.samplers.size === 0 && !entry.isLoading && entry.holds === 0) {
      this.entries.delete(key);
      this.queued.delete(key);
    }
  }
}

/**
 * @param texture - What a key holds on the GPU, or nothing.
 * @param placeholder - What its target draws without it, which says the kind of texture the target samples.
 * @returns The texture, where it is of that kind: a flat sampler bound to a sky's cube, or the reverse, would not
 *   compile, so it keeps drawing its placeholder.
 */
function toDrawn(texture: Nullable<Texture>, placeholder: Texture): Texture {
  return texture && isCube(texture) === isCube(placeholder) ? texture : placeholder;
}

/** Whether a texture is a cube, which three says by a flag its types leave to the cube classes. */
function isCube(texture: Texture): boolean {
  return (texture as Partial<CubeTexture>).isCubeTexture === true;
}
