import { Maybe, Nullable } from "@xrf/types";
import { texture as sample } from "three/tsl";
import { Node, Texture, TextureNode, WebGPURenderer } from "three/webgpu";

import { ERendererTextureEncoding, TRendererTextureSource } from "#/contract/scene/renderer-texture-source";
import { IDdsRefusal } from "#/texture/dds/dds-refusal";
import {
  createRendererImageTexture,
  createRendererRawTexture,
  createRendererTexture,
  IRendererTextureUpload,
} from "#/texture/renderer-texture";

/** One key's texture and every sampler drawing it, each with what it samples while the key holds nothing. */
interface ITextureEntry {
  /** The texture last put, which may not be on the GPU yet. */
  texture: Nullable<Texture>;
  /** What the samplers draw: always a texture already on the GPU, or nothing. */
  drawn: Nullable<Texture>;
  samplers: Map<TextureNode, Texture>;
  /** Bumped by every put and release, so a picture decoding late can tell it was superseded. */
  version: number;
  /** Whether a picture is still decoding for it. */
  isDecoding: boolean;
}

/**
 * The textures a consumer put, by key, bound into whatever samples them without recompiling anything.
 * A put texture goes up to the GPU in the frame loop, a few milliseconds a frame, and its samplers switch to it once it
 * is there: three uploads a texture the first time a binding needs it, which put a level's textures into one frame.
 */
export class RendererTextures {
  private readonly entries: Map<string, ITextureEntry> = new Map();
  /** Keys whose texture is not on the GPU yet, in the order they were put. */
  private readonly queued: Set<string> = new Set();
  private readonly onRefused: (key: string, refusal: IDdsRefusal) => void;
  private readonly onRebound: (key: string) => void;

  /**
   * @param onRefused - Told a file could not be uploaded as stored.
   * @param onRebound - Told a key's samplers were pointed at another texture, which a recorded bundle drawing them
   *   does not see until it is recorded again.
   */
  public constructor(onRefused: (key: string, refusal: IDdsRefusal) => void, onRebound: (key: string) => void) {
    this.onRefused = onRefused;
    this.onRebound = onRebound;
  }

  /**
   * @param key - What the texture is put under.
   * @param source - Its bytes.
   */
  public put(key: string, source: TRendererTextureSource): void {
    const entry: ITextureEntry = this.getEntry(key);
    const version: number = ++entry.version;

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

    entry.isDecoding = true;

    createRendererImageTexture(source.bytes, source.type)
      .then((texture: Texture) => {
        if (entry.version === version && this.entries.get(key) === entry) {
          entry.isDecoding = false;
          this.assign(key, entry, texture);
        } else {
          texture.dispose();
        }
      })
      .catch(() => {
        if (entry.version === version) {
          entry.isDecoding = false;
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
    entry.isDecoding = false;
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
    if (!key) {
      return sample(placeholder, coordinates);
    }

    const entry: ITextureEntry = this.getEntry(key);
    const sampler: TextureNode = sample(entry.drawn ?? placeholder, coordinates);

    entry.samplers.set(sampler, placeholder);

    return sampler;
  }

  /**
   * @param key - The key a sampler was bound to.
   * @param sampler - The sampler, whose material is going away.
   */
  public unbind(key: Maybe<string>, sampler: TextureNode): void {
    const entry: Maybe<ITextureEntry> = key ? this.entries.get(key) : undefined;

    if (key && entry) {
      entry.samplers.delete(sampler);
      this.prune(key, entry);
    }
  }

  /**
   * @param key - A texture's key.
   * @returns Whether what the key holds is on the GPU, so drawing with it stalls nothing; true for a key holding
   *   nothing, whose samplers draw their placeholder.
   */
  public isUploaded(key: string): boolean {
    const entry: Maybe<ITextureEntry> = this.entries.get(key);

    return !entry || (!entry.isDecoding && entry.drawn === entry.texture);
  }

  /** Whether any texture waits to go up. */
  public get hasQueued(): boolean {
    return this.queued.size > 0;
  }

  /**
   * Uploads queued textures, oldest first, until the budget is spent.
   *
   * @param renderer - The renderer uploading.
   * @param budget - Milliseconds to spend; at least one texture goes up whatever it costs.
   */
  public upload(renderer: WebGPURenderer, budget: number): void {
    const started: number = performance.now();

    for (const key of this.queued) {
      this.queued.delete(key);

      const entry: Maybe<ITextureEntry> = this.entries.get(key);

      if (!entry?.texture || entry.drawn === entry.texture) {
        continue;
      }

      renderer.initTexture(entry.texture);
      this.draw(key, entry, entry.texture);

      if (performance.now() - started >= budget) {
        return;
      }
    }
  }

  public dispose(): void {
    this.entries.forEach((entry: ITextureEntry) => {
      entry.texture?.dispose();

      if (entry.drawn !== entry.texture) {
        entry.drawn?.dispose();
      }
    });
    this.entries.clear();
    this.queued.clear();
  }

  private getEntry(key: string): ITextureEntry {
    let entry: Maybe<ITextureEntry> = this.entries.get(key);

    if (!entry) {
      entry = { drawn: null, isDecoding: false, samplers: new Map(), texture: null, version: 0 };
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

  /** Points every sampler of an entry at what it draws now, letting go of what it drew before. */
  private draw(key: string, entry: ITextureEntry, texture: Nullable<Texture>): void {
    const previous: Nullable<Texture> = entry.drawn;

    entry.drawn = texture;
    entry.samplers.forEach((placeholder: Texture, sampler: TextureNode) => (sampler.value = texture ?? placeholder));

    if (entry.samplers.size && previous !== texture) {
      this.onRebound(key);
    }

    if (previous && previous !== texture) {
      previous.dispose();
    }
  }

  /** Forgets a key nothing holds and nothing samples. */
  private prune(key: string, entry: ITextureEntry): void {
    if (!entry.texture && !entry.drawn && entry.samplers.size === 0 && !entry.isDecoding) {
      this.entries.delete(key);
      this.queued.delete(key);
    }
  }
}
