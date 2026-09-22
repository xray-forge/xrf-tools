import { Maybe, Nullable } from "@xrf/types";
import { texture as sample } from "three/tsl";
import { Node, Texture, TextureNode } from "three/webgpu";

import { ERendererTextureEncoding, TRendererTextureSource } from "#/contract/scene/renderer-texture-source";
import { IDdsRefusal } from "#/texture/dds/dds-refusal";
import { createRendererImageTexture, createRendererTexture, IRendererTextureUpload } from "#/texture/renderer-texture";

/** One key's texture and every sampler drawing it, each with what it samples while the key holds nothing. */
interface ITextureEntry {
  texture: Nullable<Texture>;
  samplers: Map<TextureNode, Texture>;
  /** Bumped by every put and release, so a picture decoding late can tell it was superseded. */
  version: number;
}

/**
 * The textures a consumer put, by key, bound into whatever samples them without recompiling anything.
 */
export class RendererTextures {
  private readonly entries: Map<string, ITextureEntry> = new Map();
  private readonly onRefused: (key: string, refusal: IDdsRefusal) => void;

  public constructor(onRefused: (key: string, refusal: IDdsRefusal) => void) {
    this.onRefused = onRefused;
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

      this.assign(entry, upload.texture);

      if (upload.refusal) {
        this.onRefused(key, upload.refusal);
      }

      return;
    }

    createRendererImageTexture(source.bytes, source.type)
      .then((texture: Texture) => {
        if (entry.version === version && this.entries.get(key) === entry) {
          this.assign(entry, texture);
        } else {
          texture.dispose();
        }
      })
      .catch(() => {
        if (entry.version === version) {
          this.assign(entry, null);
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
    this.assign(entry, null);
    this.prune(key, entry);
  }

  /**
   * A sampler of whatever the key holds, now and after every later put.
   *
   * @param key - The texture's key, or nothing for a slot the surface leaves empty.
   * @param placeholder - What it samples while the key holds nothing.
   * @param coordinates - Where it samples.
   * @returns The sampler.
   */
  public bind(key: Maybe<string>, placeholder: Texture, coordinates: Node): TextureNode {
    if (!key) {
      return sample(placeholder, coordinates);
    }

    const entry: ITextureEntry = this.getEntry(key);
    const sampler: TextureNode = sample(entry.texture ?? placeholder, coordinates);

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

  public dispose(): void {
    this.entries.forEach((entry: ITextureEntry) => entry.texture?.dispose());
    this.entries.clear();
  }

  private getEntry(key: string): ITextureEntry {
    let entry: Maybe<ITextureEntry> = this.entries.get(key);

    if (!entry) {
      entry = { samplers: new Map(), texture: null, version: 0 };
      this.entries.set(key, entry);
    }

    return entry;
  }

  private assign(entry: ITextureEntry, texture: Nullable<Texture>): void {
    if (entry.texture !== texture) {
      entry.texture?.dispose();
    }

    entry.texture = texture;
    entry.samplers.forEach((placeholder: Texture, sampler: TextureNode) => (sampler.value = texture ?? placeholder));
  }

  /** Forgets a key nothing holds and nothing samples. */
  private prune(key: string, entry: ITextureEntry): void {
    if (!entry.texture && entry.samplers.size === 0) {
      this.entries.delete(key);
    }
  }
}
