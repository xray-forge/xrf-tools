import {
  AmbientLight,
  BufferGeometry,
  DirectionalLight,
  Mesh,
  MeshStandardMaterial,
  PerspectiveCamera,
  Scene,
} from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";

import { TFrameRateLimit } from "@/core/render/lib/render-frame-limit";
import { RenderViewport } from "@/core/render/lib/render-viewport";
import {
  EMPTY_TEXTURE_SURFACE,
  ETextureSurfaceShape,
  ITextureSurfaceOptions,
  ITextureSurfaceTextures,
  listTextureSurfaceTextures,
} from "@/core/textures/lib/texture-surface";
import { applyXrayBumpShading, IVisualBumpShading } from "@/core/visuals/lib/visual-bump";
import { bindDragCursor } from "@/lib/media/drag-cursor";
import { toDolliedPosition } from "@/lib/media/orbit-dolly";
import { Nullable } from "@/lib/types/general";

import { createTextureSurfaceGeometry, toLightPosition } from "./TextureSurfaceScene.utils";

/** Where the light stands, in the angles a drag moves. */
interface ILightAngles {
  azimuth: number;
  elevation: number;
}

const INITIAL_LIGHT: ILightAngles = { azimuth: Math.PI / 4, elevation: Math.PI / 5 };

/** How far a drag across the whole viewport swings the light, in radians. */
const LIGHT_DRAG_SPEED: number = Math.PI;

/** How hard the light and the fill are driven, in the two states the switch has. */
interface ILightIntensity {
  ambient: number;
  directional: number;
}

const LIT_INTENSITY: ILightIntensity = { ambient: 0.35, directional: 2.6 };

/** A flat ambient at full strength, which reproduces the flat picture of the same file. */
const UNLIT_INTENSITY: ILightIntensity = { ambient: 1, directional: 0 };

/** Where the camera starts and returns to. */
const CAMERA_DISTANCE: number = 5;

/**
 * One texture on a lit body, shaded the way the engine shades it.
 * The canvas is transparent and the scene has no background, so the alpha checkerboard behind it is the frame's own.
 */
export class TextureSurfaceScene {
  private readonly viewport: RenderViewport;
  private readonly controls: OrbitControls;
  private readonly light: DirectionalLight;
  private readonly ambient: AmbientLight;
  private readonly material: MeshStandardMaterial;
  private readonly edgeMaterial: MeshStandardMaterial;

  /** Stops the canvas answering drags with the drag cursor, called when the scene goes. */
  private readonly unbindDragCursor: () => void;

  private mesh: Nullable<Mesh<BufferGeometry, MeshStandardMaterial | Array<MeshStandardMaterial>>> = null;
  private shading: Nullable<IVisualBumpShading> = null;
  private textures: ITextureSurfaceTextures = EMPTY_TEXTURE_SURFACE;
  private options: ITextureSurfaceOptions = {
    isBumped: true,
    isLit: true,
    shape: ETextureSurfaceShape.PLANE,
    tiling: 1,
  };

  private lightAngles: ILightAngles = { ...INITIAL_LIGHT };

  public constructor() {
    // No background colour, so the canvas is transparent and the checkerboard the frame already draws shows through
    // wherever the texture's alpha does.
    this.viewport = new RenderViewport(
      { backgroundColor: null, cameraFar: 100, cameraFieldOfView: 45, cameraNear: 0.01 },
      { onFrame: () => this.controls.update() }
    );
    this.camera.position.set(0, 0, CAMERA_DISTANCE);

    this.controls = new OrbitControls(this.camera, this.viewport.domElement);
    this.controls.enableDamping = true;

    this.light = new DirectionalLight(0xffffff, LIT_INTENSITY.directional);
    this.light.position.copy(toLightPosition(this.lightAngles.azimuth, this.lightAngles.elevation));

    // Low rather than absent, so the unlit side of a sphere is readable without washing the bump response out.
    this.ambient = new AmbientLight(0xffffff, LIT_INTENSITY.ambient);

    this.scene.add(this.ambient);
    this.scene.add(this.light);

    this.material = new MeshStandardMaterial({ metalness: 0, roughness: 1 });
    // Dark and plain, so a rotated slab reads as a slab: its four edges and its back are not the texture.
    this.edgeMaterial = new MeshStandardMaterial({ color: 0x1a1a1a, metalness: 0, roughness: 0.9 });

    this.setShape(this.options.shape);
    this.unbindDragCursor = bindDragCursor(this.controls, this.viewport.domElement);
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
   * Puts the canvas on screen and starts drawing.
   *
   * @param container - Element the canvas fills.
   */
  public mount(container: HTMLElement): void {
    this.viewport.mount(container);
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
      // Unpatched rather than switched off: a texture declaring no pair must compile the stock program, or it keeps
      // sampling the last texture's bump through a patch nothing is left to disable.
      this.shading = null;
      this.material.onBeforeCompile = () => undefined;
      this.material.customProgramCacheKey = () => "";
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
   * Swings the light by a drag across the viewport.
   *
   * @param deltaX - Horizontal movement in pixels.
   * @param deltaY - Vertical movement in pixels.
   */
  public dragLight(deltaX: number, deltaY: number): void {
    const width: number = this.viewport.width || 1;
    const height: number = this.viewport.height || 1;
    const limit: number = Math.PI / 2 - 0.05;

    this.lightAngles = {
      azimuth: this.lightAngles.azimuth + (deltaX / width) * LIGHT_DRAG_SPEED,
      // Clamped short of the poles, where a directional light stops telling a bumped surface from a flat one.
      elevation: Math.max(-limit, Math.min(limit, this.lightAngles.elevation - (deltaY / height) * LIGHT_DRAG_SPEED)),
    };

    this.light.position.copy(toLightPosition(this.lightAngles.azimuth, this.lightAngles.elevation));
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

  /** Puts the camera and the light back where they started. */
  public reset(): void {
    this.camera.position.set(0, 0, CAMERA_DISTANCE);
    this.controls.target.set(0, 0, 0);
    this.controls.update();

    this.lightAngles = { ...INITIAL_LIGHT };
    this.light.position.copy(toLightPosition(this.lightAngles.azimuth, this.lightAngles.elevation));
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

    const intensity: ILightIntensity = this.options.isLit ? LIT_INTENSITY : UNLIT_INTENSITY;

    this.light.intensity = intensity.directional;
    this.ambient.intensity = intensity.ambient;

    this.applyTiling();
    this.applyAspect();
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
