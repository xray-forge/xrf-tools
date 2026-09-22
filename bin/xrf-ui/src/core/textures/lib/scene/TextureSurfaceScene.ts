import {
  bindDragCursor,
  IRenderFrameCost,
  IRenderTarget,
  TFrameRateLimit,
  toDolliedPosition,
  TRenderInputElement,
} from "@xrf/renderer";
import { Nullable } from "@xrf/types";
import { BufferGeometry, Matrix3, Mesh, MeshStandardMaterial, PerspectiveCamera, Scene, Texture } from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";

import { TRenderCostReporter } from "@/core/render/lib/frame/render-reporter";
import { RenderViewport } from "@/core/render/lib/frame/render-viewport";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { RenderPreviewLighting } from "@/core/render/lib/lighting/RenderPreviewLighting";
import { applyXrayGlossShading } from "@/core/render/lib/surface/render-gloss";
import { XRAY_DEFAULT_AREF } from "@/core/render/lib/surface/render-surface";
import {
  createDdsTexture,
  createDecodedTexture,
  hasRenderTextureAlpha,
} from "@/core/render/lib/texture/render-texture";
import {
  EMPTY_TEXTURE_SURFACE,
  ETextureSurfaceAlpha,
  ETextureSurfaceShape,
  ITextureSurfaceFile,
  ITextureSurfaceFiles,
  ITextureSurfaceOptions,
} from "@/core/textures/lib/texture-surface";
import {
  applyXrayBumpShading,
  IVisualBumpShading,
  IVisualBumpTextures,
  removeXrayBumpShading,
} from "@/core/visuals/lib/visual-bump";

import {
  DEFAULT_TEXTURE_LIGHTING,
  TEXTURE_LIGHT_DRAG_SPEED,
  TEXTURE_LIGHT_ELEVATION_LIMIT,
  UNLIT_TEXTURE_LIGHTING,
} from "./texture-lighting";
import { createTextureSurfaceGeometry } from "./TextureSurfaceScene.utils";

/** Where the camera starts and returns to. */
const CAMERA_DISTANCE: number = 5;

/**
 * One texture on a lit body, shaded the way the engine shades it.
 * The canvas is transparent and the scene has no background, so the alpha checkerboard behind it is the frame's own.
 */
export class TextureSurfaceScene {
  private readonly viewport: RenderViewport;
  private readonly controls: OrbitControls;
  private readonly lights: RenderPreviewLighting;
  private readonly material: MeshStandardMaterial;
  private readonly edgeMaterial: MeshStandardMaterial;

  /** Stops the canvas answering drags with the drag cursor, called when the scene goes. */
  private readonly unbindDragCursor: () => void;

  /** Told what frames are costing, or nothing while nobody is reading them. */
  private reporter: Nullable<TRenderCostReporter> = null;

  private mesh: Nullable<Mesh<BufferGeometry, MeshStandardMaterial | Array<MeshStandardMaterial>>> = null;
  private shading: Nullable<IVisualBumpShading> = null;
  private files: ITextureSurfaceFiles = EMPTY_TEXTURE_SURFACE;
  /** What this scene uploaded and is therefore the one to release. */
  private uploaded: Array<Texture> = [];
  /** Which set the uploads belong to, so a decode that lands after the next texture is chosen is dropped. */
  private uploading: number = 0;

  private options: ITextureSurfaceOptions = {
    alpha: ETextureSurfaceAlpha.CUT_OUT,
    isBumped: true,
    isLit: true,
    shape: ETextureSurfaceShape.PLANE,
    tiling: 1,
  };

  /** What the surface is lit with, which its owner holds and a drag over the body reports back. */
  private lighting: IRenderLighting = DEFAULT_TEXTURE_LIGHTING;

  public constructor(target: IRenderTarget, element: TRenderInputElement) {
    // No background colour, so the canvas is transparent and the checkerboard the frame already draws shows through
    // wherever the texture's alpha does.
    this.viewport = new RenderViewport(
      target,
      { backgroundColor: null, cameraFar: 100, cameraFieldOfView: 45, cameraNear: 0.01 },
      {
        onFrame: () => this.controls.update(),
        onReport: (cost: IRenderFrameCost) => this.reporter?.(cost),
      }
    );
    this.camera.position.set(0, 0, CAMERA_DISTANCE);

    // Cast because three types an element it only ever listens to, measures and writes a cursor on - which is
    // exactly what a stand-in for one answers.
    this.controls = new OrbitControls(this.camera, element as HTMLElement);
    this.controls.enableDamping = true;

    this.lights = new RenderPreviewLighting(this.scene, DEFAULT_TEXTURE_LIGHTING);

    this.material = new MeshStandardMaterial({ metalness: 0, roughness: 1 });
    // The same gloss rule the rest of the viewer draws by: nothing X-Ray ships is reflective until its bump says so,
    // and the patch below writes the pair's own gloss over this default wherever one is bound.
    applyXrayGlossShading(this.material);
    // Dark and plain, so a rotated slab reads as a slab: its four edges and its back are not the texture.
    this.edgeMaterial = new MeshStandardMaterial({ color: 0x1a1a1a, metalness: 0, roughness: 0.9 });

    this.setShape(this.options.shape);
    this.unbindDragCursor = bindDragCursor(this.controls, element);
  }

  /** The scene the body stands in, which the viewport draws. */
  private get scene(): Scene {
    return this.viewport.scene;
  }

  /** The camera the orbit controls drive. */
  private get camera(): PerspectiveCamera {
    return this.viewport.camera;
  }

  /**
   * Takes what to tell about frame cost, or nothing to stop telling.
   *
   * @param reporter - Told what frames are costing, a few times a second.
   */
  public setReporter(reporter: Nullable<TRenderCostReporter>): void {
    this.reporter = reporter;
  }

  /**
   * Caps how often the scene redraws.
   *
   * @param limit - Frames a second to allow, as the application setting states it.
   */
  public setFrameRateLimit(limit: TFrameRateLimit): void {
    this.viewport.setFrameRateLimit(limit);
  }

  /**
   * Takes the canvas off screen and releases everything it holds.
   */
  public dispose(): void {
    this.releaseUploads();
    this.unbindDragCursor();
    this.controls.dispose();
    this.mesh?.geometry.dispose();
    this.material.dispose();
    this.edgeMaterial.dispose();
    this.lights.dispose();
    this.viewport.dispose();
  }

  /**
   * Lays a different body under the same texture.
   *
   * @param shape - Body to draw.
   */
  public setShape(shape: ETextureSurfaceShape): void {
    this.options = { ...this.options, shape };

    if (this.mesh) {
      this.scene.remove(this.mesh);
      this.mesh.geometry.dispose();
    }

    this.mesh = new Mesh(createTextureSurfaceGeometry(shape), this.toShapeMaterials(shape));
    this.scene.add(this.mesh);

    this.applyAspect();
  }

  /**
   * Draws a different texture, with the pair its descriptor binds.
   *
   * @param files - The base file and the pair, as they were read.
   */
  public setTextures(files: ITextureSurfaceFiles): void {
    const generation: number = (this.uploading += 1);

    this.files = files;

    this.releaseUploads();

    const base: Nullable<Texture> = files.base && !files.base.isDecoded ? this.upload(files.base, true) : null;
    const bump: Nullable<IVisualBumpTextures> = files.bump
      ? { bump: this.upload(files.bump.bump), companion: this.upload(files.bump.companion) }
      : null;

    this.dress(base, bump);

    // A layout three.js refuses arrives as the backend's picture instead, and decoding one is asynchronous.
    if (files.base?.isDecoded) {
      void this.uploadDecoded(files.base, generation, bump);
    }
  }

  /**
   * Puts what was uploaded onto the material.
   *
   * @param base - The base texture, or null while there is none to draw.
   * @param bump - The pair, or null for a material that binds none.
   */
  private dress(base: Nullable<Texture>, bump: Nullable<IVisualBumpTextures>): void {
    this.material.map = base;

    if (bump) {
      // Re-patched per pair rather than kept, because the patch closes over the two samplers it was given.
      this.shading = applyXrayBumpShading(this.material, bump);
    } else {
      // Taken off rather than switched off: a texture declaring no pair must compile the stock program, or it keeps
      // sampling the last texture's bump through a patch nothing is left to disable.
      this.shading = null;
      removeXrayBumpShading(this.material);
    }

    this.applyOptions();

    this.material.needsUpdate = true;
  }

  /**
   * Uploads one file, keeping it to release later.
   *
   * @param file - The file as it was read.
   * @param isColor - Whether it holds srgb values rather than packed ones.
   * @returns The texture, or null for a layout this side will not upload.
   */
  private upload(file: ITextureSurfaceFile, isColor: boolean = false): Texture {
    // Colour, not data, for a base texture: saying so is what makes the unlit body match the flat picture of the
    // same file. A pair is packed values and must not be told otherwise.
    const texture: Nullable<Texture> = createDdsTexture(file.bytes, { isAlphaRead: isColor, isColor }).texture;

    if (texture) {
      this.uploaded.push(texture);
    }

    return texture as Texture;
  }

  /**
   * Uploads the backend's picture of a layout three.js refused, and draws it if it is still the one wanted.
   *
   * @param file - The decoded file.
   * @param generation - Which set it belongs to.
   * @param bump - The pair uploaded beside it.
   */
  private async uploadDecoded(
    file: ITextureSurfaceFile,
    generation: number,
    bump: Nullable<IVisualBumpTextures>
  ): Promise<void> {
    const texture: Texture = await createDecodedTexture(file.bytes, { isColor: true });

    // Another texture was chosen while this decoded, and its uploads are already on the material.
    if (generation !== this.uploading) {
      texture.dispose();

      return;
    }

    this.uploaded.push(texture);
    this.dress(texture, bump);
  }

  /** Releases what this scene uploaded, which nothing else holds. */
  private releaseUploads(): void {
    for (const texture of this.uploaded) {
      texture.dispose();
    }

    this.uploaded = [];
  }

  /**
   * Applies how the surface is being looked at.
   *
   * @param options - Shape, lighting, bump switch and tiling.
   */
  public setOptions(options: ITextureSurfaceOptions): void {
    if (options.shape !== this.options.shape) {
      this.setShape(options.shape);
    }

    this.options = options;

    this.applyOptions();
  }

  /**
   * Takes what the surface is lit with.
   *
   * @param lighting - The direction, its strength and colour, and the fill.
   */
  public setLighting(lighting: IRenderLighting): void {
    this.lighting = lighting;

    this.applyLighting();
  }

  /**
   * Swings the light by a drag across the viewport.
   *
   * @param deltaX - Horizontal movement in pixels.
   * @param deltaY - Vertical movement in pixels.
   * @returns Where the drag has put the light.
   */
  public dragLight(deltaX: number, deltaY: number): IRenderLighting {
    const width: number = this.viewport.width || 1;
    const height: number = this.viewport.height || 1;
    const elevation: number = this.lighting.sunElevation - (deltaY / height) * TEXTURE_LIGHT_DRAG_SPEED;

    return {
      ...this.lighting,
      sunAzimuth: this.lighting.sunAzimuth + (deltaX / width) * TEXTURE_LIGHT_DRAG_SPEED,
      sunElevation: Math.max(-TEXTURE_LIGHT_ELEVATION_LIMIT, Math.min(TEXTURE_LIGHT_ELEVATION_LIMIT, elevation)),
    };
  }

  /**
   * Moves the camera along the line it is looking down, by one notch of the shared step.
   *
   * @param step - Multiplier on the distance to what the camera orbits; above one moves away.
   */
  public dolly(step: number): void {
    const { x, y, z } = this.camera.position;
    const target = this.controls.target;

    this.camera.position.set(
      ...toDolliedPosition(
        [x, y, z],
        [target.x, target.y, target.z],
        step,
        this.controls.minDistance,
        this.controls.maxDistance
      )
    );

    this.controls.update();
  }

  /** Puts the camera back where it started. The light is its owner's, and the panel offering it puts that back. */
  public reset(): void {
    this.camera.position.set(0, 0, CAMERA_DISTANCE);
    this.controls.target.set(0, 0, 0);
    this.controls.update();
  }

  /**
   * The materials a body is drawn with, one per geometry group.
   */
  private toShapeMaterials(shape: ETextureSurfaceShape): MeshStandardMaterial | Array<MeshStandardMaterial> {
    if (shape !== ETextureSurfaceShape.PLANE) {
      return this.material;
    }

    // `BoxGeometry` groups its faces +x, -x, +y, -y, +z, -z, and the texture belongs on the one facing the camera.
    return [
      this.edgeMaterial,
      this.edgeMaterial,
      this.edgeMaterial,
      this.edgeMaterial,
      this.material,
      this.edgeMaterial,
    ];
  }

  /** Applies everything the current options say, from whichever of them changed. */
  private applyOptions(): void {
    // Nothing to shade without a light: under a flat ambient the decoded normal changes no pixel, so the switch says
    // so rather than pretending the surface is still being compared.
    this.shading?.setEnabled(this.options.isBumped && this.options.isLit);

    this.applyAlpha();
    this.applyLighting();
    this.applyTiling();
    this.applyAspect();
  }

  /**
   * Reads the alpha channel the way the chosen shader would, or leaves it unread the way the plain one does.
   */
  private applyAlpha(): void {
    const isRead: boolean = hasRenderTextureAlpha(this.material.map);
    const alpha: ETextureSurfaceAlpha = isRead ? this.options.alpha : ETextureSurfaceAlpha.IGNORED;
    const alphaTest: number = alpha === ETextureSurfaceAlpha.CUT_OUT ? XRAY_DEFAULT_AREF : 0;
    // Still in the opaque pass when it is cut out, which is where the engine's `clip` happens: only the forward
    // blenders composite, and those are the third answer rather than a softer version of the second.
    const isTransparent: boolean = alpha === ETextureSurfaceAlpha.BLENDED;

    // Both of these change the compiled program, so a change to either asks for it - and only a change does, or
    // every drag of the light slider would recompile the shader the body is drawn with.
    if (this.material.alphaTest === alphaTest && this.material.transparent === isTransparent) {
      return;
    }

    this.material.alphaTest = alphaTest;
    this.material.transparent = isTransparent;
    this.material.needsUpdate = true;
  }

  /** The light the body actually stands under, which the switch turns over to a flat fill and back. */
  private applyLighting(): void {
    this.lights.apply(this.options.isLit ? this.lighting : UNLIT_TEXTURE_LIGHTING);
  }

  /**
   * Stretches a flat body to the proportions of the texture on it.
   *
   * Only the plane: a sphere and a cube are bodies in their own right, and squashing one to a texture's proportions
   * would answer a question nobody asked, while a plane is the texture and nothing else.
   */
  private applyAspect(): void {
    if (!this.mesh) {
      return;
    }

    const aspect: number = this.options.shape === ETextureSurfaceShape.PLANE ? this.files.aspect : 1;

    // Depth is left alone, so a slab keeps one thickness whatever proportions the texture on it has.
    this.mesh.scale.set(aspect >= 1 ? 1 : aspect, aspect >= 1 ? 1 / aspect : 1, 1);
  }

  /** Repeats every texture the surface samples, the base through three.js and the pair through the patch. */
  private applyTiling(): void {
    for (const texture of this.uploaded) {
      texture.repeat.set(this.options.tiling, this.options.tiling);
      texture.updateMatrix();
    }

    // The patch samples the pair itself, so nothing else would carry the repeat to it: without this the base tiles
    // and the bump does not, and one tile of detail is shaded across every tile of colour.
    if (this.shading) {
      this.shading.setUvTransform(this.bumpMatrix());
    }
  }

  /**
   * @returns The uv transform the pair is sampled through, which is the first half's.
   */
  private bumpMatrix(): Matrix3 {
    return (this.uploaded.at(-2) ?? this.uploaded[0]).matrix;
  }
}
