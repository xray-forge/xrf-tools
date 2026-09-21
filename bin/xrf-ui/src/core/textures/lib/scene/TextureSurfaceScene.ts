import { BufferGeometry, Mesh, MeshStandardMaterial, PerspectiveCamera, Scene } from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";

import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import { TFrameRateLimit } from "@/core/render/lib/frame/render-frame-limit";
import { RenderViewport } from "@/core/render/lib/frame/render-viewport";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { RenderPreviewLighting } from "@/core/render/lib/lighting/RenderPreviewLighting";
import { applyXrayGlossShading } from "@/core/render/lib/surface/render-gloss";
import { XRAY_DEFAULT_AREF } from "@/core/render/lib/surface/render-surface";
import { hasRenderTextureAlpha } from "@/core/render/lib/texture/render-texture";
import {
  EMPTY_TEXTURE_SURFACE,
  ETextureSurfaceAlpha,
  ETextureSurfaceShape,
  ITextureSurfaceOptions,
  ITextureSurfaceTextures,
  listTextureSurfaceTextures,
} from "@/core/textures/lib/texture-surface";
import { applyXrayBumpShading, IVisualBumpShading, removeXrayBumpShading } from "@/core/visuals/lib/visual-bump";
import { bindDragCursor } from "@/lib/media/drag-cursor";
import { toDolliedPosition } from "@/lib/media/orbit-dolly";
import { Nullable } from "@/lib/types/general";

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

  private mesh: Nullable<Mesh<BufferGeometry, MeshStandardMaterial | Array<MeshStandardMaterial>>> = null;
  private shading: Nullable<IVisualBumpShading> = null;
  private textures: ITextureSurfaceTextures = EMPTY_TEXTURE_SURFACE;
  private options: ITextureSurfaceOptions = {
    alpha: ETextureSurfaceAlpha.CUT_OUT,
    isBumped: true,
    isLit: true,
    shape: ETextureSurfaceShape.PLANE,
    tiling: 1,
  };

  /** What the surface is lit with, which its owner holds and a drag over the body reports back. */
  private lighting: IRenderLighting = DEFAULT_TEXTURE_LIGHTING;

  public constructor(target: DomRenderTarget) {
    // No background colour, so the canvas is transparent and the checkerboard the frame already draws shows through
    // wherever the texture's alpha does.
    this.viewport = new RenderViewport(
      target,
      { backgroundColor: null, cameraFar: 100, cameraFieldOfView: 45, cameraNear: 0.01 },
      { onFrame: () => this.controls.update() }
    );
    this.camera.position.set(0, 0, CAMERA_DISTANCE);

    this.controls = new OrbitControls(this.camera, target.canvas);
    this.controls.enableDamping = true;

    this.lights = new RenderPreviewLighting(this.scene, DEFAULT_TEXTURE_LIGHTING);

    this.material = new MeshStandardMaterial({ metalness: 0, roughness: 1 });
    // The same gloss rule the rest of the viewer draws by: nothing X-Ray ships is reflective until its bump says so,
    // and the patch below writes the pair's own gloss over this default wherever one is bound.
    applyXrayGlossShading(this.material);
    // Dark and plain, so a rotated slab reads as a slab: its four edges and its back are not the texture.
    this.edgeMaterial = new MeshStandardMaterial({ color: 0x1a1a1a, metalness: 0, roughness: 0.9 });

    this.setShape(this.options.shape);
    this.unbindDragCursor = bindDragCursor(this.controls, target.canvas);
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
   * @param textures - The uploaded base and pair.
   */
  public setTextures(textures: ITextureSurfaceTextures): void {
    this.textures = textures;
    this.material.map = textures.base;

    if (textures.bump) {
      // Re-patched per pair rather than kept, because the patch closes over the two samplers it was given.
      this.shading = applyXrayBumpShading(this.material, textures.bump);
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
    const isRead: boolean = hasRenderTextureAlpha(this.textures.base);
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

    const aspect: number = this.options.shape === ETextureSurfaceShape.PLANE ? this.textures.aspect : 1;

    // Depth is left alone, so a slab keeps one thickness whatever proportions the texture on it has.
    this.mesh.scale.set(aspect >= 1 ? 1 : aspect, aspect >= 1 ? 1 / aspect : 1, 1);
  }

  /** Repeats every texture the surface samples, the base through three.js and the pair through the patch. */
  private applyTiling(): void {
    for (const texture of listTextureSurfaceTextures(this.textures)) {
      texture.repeat.set(this.options.tiling, this.options.tiling);
      texture.updateMatrix();
    }

    // The patch samples the pair itself, so nothing else would carry the repeat to it: without this the base tiles
    // and the bump does not, and one tile of detail is shaded across every tile of colour.
    if (this.textures.bump) {
      this.shading?.setUvTransform(this.textures.bump.bump.matrix);
    }
  }
}
