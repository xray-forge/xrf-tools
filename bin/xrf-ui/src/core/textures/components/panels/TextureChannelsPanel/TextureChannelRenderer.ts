import { Mesh, OrthographicCamera, PlaneGeometry, Scene, WebGLRenderer } from "three";

import { IVisualBumpTextures } from "@/core/visuals/lib/visual-bump";
import { createXrayBumpChannels, EVisualBumpView, IVisualBumpChannels } from "@/core/visuals/lib/visual-bump-channels";
import { Nullable } from "@/lib/types/general";

/** The quad fills a camera of exactly its own size, so every tile is the plane and nothing else. */
const QUAD_EXTENT: number = 1;

/**
 * Draws the planes of one bump pair into as many tiles as a panel asks for.
 *
 * One renderer for every tile rather than one each, because a browser will only keep a dozen or so webgl contexts and
 * a panel of five tiles beside a lit surface would spend half of them on pictures that never change. Each tile is a
 * plain 2d canvas the shared renderer is copied into, which is three.js' own answer to the same problem.
 *
 * Nothing here animates: a tile is drawn when the pair changes, when the view it shows changes, or when it resizes.
 */
export class TextureChannelRenderer {
  private readonly renderer: WebGLRenderer;
  private readonly scene: Scene;
  private readonly camera: OrthographicCamera;
  private readonly geometry: PlaneGeometry;

  private channels: Nullable<IVisualBumpChannels> = null;
  private mesh: Nullable<Mesh> = null;

  public constructor() {
    this.renderer = new WebGLRenderer({ antialias: false });
    this.scene = new Scene();
    this.geometry = new PlaneGeometry(QUAD_EXTENT, QUAD_EXTENT);

    const half: number = QUAD_EXTENT / 2;

    this.camera = new OrthographicCamera(-half, half, half, -half, 0, 1);
    this.camera.position.z = 1;
  }

  /**
   * Points every tile at a different pair, or at none.
   *
   * @param textures - The uploaded pair, owned by whoever loaded it.
   */
  public setTextures(textures: Nullable<IVisualBumpTextures>): void {
    this.releaseChannels();

    if (!textures) {
      return;
    }

    this.channels = createXrayBumpChannels(textures);
    this.mesh = new Mesh(this.geometry, this.channels.material);

    this.scene.add(this.mesh);
  }

  /**
   * Draws one view at the size of one tile and copies it there.
   *
   * @param view - Plane to draw.
   * @param tile - Canvas to copy into, sized in css pixels by its own layout.
   * @returns Whether anything was drawn, which is false while no pair is bound.
   */
  public draw(view: EVisualBumpView, tile: HTMLCanvasElement): boolean {
    const context: Nullable<CanvasRenderingContext2D> = tile.getContext("2d");
    const width: number = tile.clientWidth;
    const height: number = tile.clientHeight;

    if (!this.channels || !context || !width || !height) {
      return false;
    }

    const ratio: number = window.devicePixelRatio;

    tile.width = Math.round(width * ratio);
    tile.height = Math.round(height * ratio);

    this.channels.setView(view);
    this.renderer.setPixelRatio(ratio);
    this.renderer.setSize(width, height, false);
    this.renderer.render(this.scene, this.camera);

    // Copied in the same turn as the render, before the drawing buffer is presented and cleared, which is what lets
    // this renderer be shared rather than preserved.
    context.drawImage(this.renderer.domElement, 0, 0, tile.width, tile.height);

    return true;
  }

  /** Releases everything the renderer owns. The pair is not its own and is left alone. */
  public dispose(): void {
    this.releaseChannels();
    this.geometry.dispose();
    this.renderer.dispose();
  }

  private releaseChannels(): void {
    if (this.mesh) {
      this.scene.remove(this.mesh);
      this.mesh = null;
    }

    this.channels?.dispose();
    this.channels = null;
  }
}
