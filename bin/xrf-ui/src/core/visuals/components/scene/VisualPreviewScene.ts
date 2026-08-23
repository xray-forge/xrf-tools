import {
  AmbientLight,
  AxesHelper,
  Bone,
  BufferAttribute,
  BufferGeometry,
  Color,
  DataTexture,
  DirectionalLight,
  GridHelper,
  LineBasicMaterial,
  LineSegments,
  Matrix4,
  Mesh,
  MeshStandardMaterial,
  PerspectiveCamera,
  Points,
  PointsMaterial,
  Scene,
  Skeleton,
  SkinnedMesh,
  Texture,
  WebGLRenderer,
} from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";

import {
  DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG,
  IVisualPreviewSceneConfig,
} from "@/core/visuals/components/scene/scene-config";
import { createCheckerTexture, createSubmeshGeometry } from "@/core/visuals/components/scene/VisualPreviewScene.utils";
import {
  FLOATS_PER_BONE,
  getVisualSubmeshLevel,
  IVisualModelViews,
  IVisualSubmeshLevel,
  TRANSLATION_OFFSET,
} from "@/core/visuals/lib/visual-views";
import { Nullable, Optional } from "@/lib/types/general";

/**
 * How a preview looks before anyone touches a toggle.
 */
export const DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS: IVisualPreviewViewOptions = {
  isWireframe: false,
  isGridVisible: true,
  isAxesVisible: true,
  isCheckerVisible: false,
  isSkeletonVisible: false,
};

/** Radius assumed when a model reports no usable extent, so the camera and helpers still have a scale. */
const FALLBACK_RADIUS: number = 1;

/**
 * View state the toolbar owns and the scene applies.
 *
 * React holds the state, the scene stays a sink: it never reports view state back, so there is one source of truth and
 * no synchronisation between react and the scene graph. Distinct from the scene's configuration, which is chosen once
 * and describes how the preview looks rather than what the user is toggling.
 */
export interface IVisualPreviewViewOptions {
  isWireframe: boolean;
  isGridVisible: boolean;
  isAxesVisible: boolean;
  /**
   * Renders a repeating checkerboard from the uv buffer instead of a flat surface.
   *
   * Present before textures are: it is the only way to see that the v flip is right, which otherwise stays invisible
   * until textures land and come out mirrored.
   */
  isCheckerVisible: boolean;
  /**
   * Draws the bind pose over the mesh.
   *
   * Rendered with depth testing off, because a skeleton is only useful when it can be seen through the mesh it sits
   * inside - which is the point when checking where a weapon's attach bone actually is.
   */
  isSkeletonVisible: boolean;
}

/**
 * One drawn submesh: its mesh, its own material, and the texture applied to it.
 */
interface IVisualSubmeshMesh {
  mesh: Mesh<BufferGeometry, MeshStandardMaterial>;
  texture: Nullable<Texture>;
}

/**
 * Owns the three.js scene imperatively, outside of react state.
 *
 * An editor scene graph is long lived and mutated by direct manipulation, so it is deliberately not expressed as react
 * elements: react only mounts it into a container and disposes it again. Everything webgl touches stays behind this
 * class.
 */
export class VisualPreviewScene {
  private readonly config: IVisualPreviewSceneConfig;
  private readonly scene: Scene;
  private readonly camera: PerspectiveCamera;
  private readonly renderer: WebGLRenderer;
  private readonly controls: OrbitControls;
  private readonly checker: DataTexture;
  private readonly grid: GridHelper;
  private readonly axes: AxesHelper;
  private readonly resizeObserver: ResizeObserver;

  /**
   * Meshes keyed by the submesh index they were built from.
   *
   * Keyed rather than ordered because textures arrive addressed by that index, out of order and after the fact, so a
   * position in an array would be the wrong thing to trust.
   */
  private meshes: Map<number, IVisualSubmeshMesh> = new Map();
  /** The bind pose overlay of the current model, or null when it carries no bind data. */
  private skeleton: Nullable<LineSegments<BufferGeometry, LineBasicMaterial>> = null;
  /** The marker for a joint named elsewhere, kept across models rather than rebuilt. */
  private highlight: Nullable<Points<BufferGeometry, PointsMaterial>> = null;
  /** Where the marker points, kept so a toggle can show it again without the selection being sent a second time. */
  private highlightedJoint: Nullable<[number, number, number]> = null;
  /**
   * One bone per bind transform, in bone order, and the skeleton the skinned meshes are bound to.
   *
   * Flat rather than parented: the backend already composed every bone into model space, so a hierarchy here would
   * compose it a second time. Their matrices are set directly and never derived, which is why they carry
   * `matrixAutoUpdate = false`.
   */
  private bones: Array<Bone> = [];
  private skin: Nullable<Skeleton> = null;
  /** The last options applied, so a texture landing later knows whether the checker is currently covering it. */
  private viewOptions: Nullable<IVisualPreviewViewOptions> = null;
  private model: Nullable<IVisualModelViews> = null;
  /**
   * How far down its collapse chain every mesh is currently drawing, 0 being full detail.
   *
   * Held here so a model replaced while detail is reduced arrives reduced, rather than snapping back to full and
   * leaving the toolbar saying otherwise.
   */
  private detail: number = 0;
  private container: Nullable<HTMLElement> = null;
  private frameHandle: number = 0;
  private isResizePending: boolean = false;
  private renderedWidth: number = 0;
  private renderedHeight: number = 0;

  public constructor(
    model: Nullable<IVisualModelViews>,
    config: IVisualPreviewSceneConfig = DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG
  ) {
    this.config = config;

    this.renderer = new WebGLRenderer({ antialias: true });
    this.renderer.setPixelRatio(window.devicePixelRatio);
    this.renderer.domElement.style.display = "block";

    this.scene = new Scene();
    this.scene.background = new Color(config.backgroundColor);

    this.camera = new PerspectiveCamera(config.cameraFieldOfView, 1, 0.001, 10000);

    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.enableDamping = true;

    this.checker = createCheckerTexture(config);
    this.grid = new GridHelper(10, 10, config.gridColor, config.gridColor);
    this.axes = new AxesHelper(1);

    const light: DirectionalLight = new DirectionalLight(0xffffff, 2);

    light.position.set(3, 5, 4);

    this.scene.add(new AmbientLight(0xffffff, 1.4));
    this.scene.add(light);
    this.scene.add(this.grid);
    this.scene.add(this.axes);

    this.resizeObserver = new ResizeObserver(() => this.resize());

    this.setModel(model);
  }

  /**
   * Replace whatever is on screen with a different model, or with nothing.
   *
   * Geometry is rebuilt while the renderer and controls survive. The camera is fitted only when there was nothing on
   * screen to have framed: stepping through a tree is comparing models, and a refit per model throws away the angle and
   * the distance the comparison is being made from. A model of a very different size can land off frame that way, which
   * is what the toolbar's reset is for.
   *
   * @param model - Model views to display, or `null` to clear the scene.
   */
  public setModel(model: Nullable<IVisualModelViews>): void {
    const hadModel: boolean = Boolean(this.model);

    this.clearModel();

    this.model = model;

    this.applySkin(model);

    for (const submesh of model?.submeshes ?? []) {
      const geometry: BufferGeometry = createSubmeshGeometry(submesh, this.detail);
      const material: MeshStandardMaterial = new MeshStandardMaterial({
        color: this.config.meshColor,
        metalness: 0.05,
        roughness: 0.75,
      });

      // Skinned only when this submesh carries links and the model carries bones to bind them to. A skinned mesh is
      // never frustum culled, because three.js measures it against its bind pose and a motion reaches outside that.
      const isSkinned: boolean = Boolean(submesh.skinIndices && submesh.skinWeights && this.skin);
      const mesh: Mesh<BufferGeometry, MeshStandardMaterial> = isSkinned
        ? new SkinnedMesh(geometry, material)
        : new Mesh(geometry, material);

      mesh.name = submesh.label;

      if (mesh instanceof SkinnedMesh && this.skin) {
        mesh.frustumCulled = false;
        // An identity bind matrix, because the vertices and the bone transforms are already in the same space: the
        // backend composed both into model space. Letting three.js take the mesh's own world matrix instead would work
        // only as long as nothing ever moved the mesh.
        mesh.bind(this.skin, new Matrix4());
      }

      this.meshes.set(submesh.index, { mesh, texture: null });
      this.scene.add(mesh);
    }

    this.applySkeleton(model);

    if (this.viewOptions) {
      this.applyViewOptions(this.viewOptions);
    }

    this.applyScale();

    if (!hadModel) {
      this.resetCamera();
    }
  }

  /**
   * Draw every mesh at a different point along its collapse chain.
   *
   * Only a draw range changes: all levels are already in the uploaded index buffer, so this touches no attribute and
   * costs no upload — which is what makes dragging the control smooth on a model carrying nine hundred levels.
   * Bounding spheres are left alone deliberately: they describe the same geometry, and refitting the camera on every
   * step would make comparing detail impossible.
   *
   * @param detail - How far down each chain to go: 0 is full detail, 1 is the coarsest each submesh has.
   */
  public setDetailLevel(detail: number): void {
    this.detail = detail;

    for (const submesh of this.model?.submeshes ?? []) {
      const drawn: Nullable<IVisualSubmeshMesh> = this.meshes.get(submesh.index) ?? null;

      if (drawn) {
        const level: IVisualSubmeshLevel = getVisualSubmeshLevel(submesh, detail);

        drawn.mesh.geometry.setDrawRange(level.start, level.count);
      }
    }
  }

  /**
   * Poses the model from one frame of a baked motion, or returns it to its bind pose.
   *
   * Drives the mesh and the overlay from one buffer: each bone's transform becomes that bone's matrix, which is what
   * skinning multiplies by the inverse bind, and its translation becomes the overlay's segment endpoints. A frame is
   * therefore a scatter of writes into matrices already allocated and one attribute update - no geometry rebuilt, no
   * buffer allocated - which is what keeps thirty frames a second from becoming thirty uploads.
   *
   * Passing `null`, or a buffer too short for the frame asked for, restores the bind pose rather than posing a
   * skeleton from whatever happens to be at that offset.
   *
   * @param transforms - Every frame's bone transforms, frame major, or null to show the bind pose again.
   * @param frame - Which frame of that buffer to show.
   * @param floatsPerBone - Floats one bone occupies, as the bake reported it.
   */
  public setPose(transforms: Nullable<Float32Array>, frame: number, floatsPerBone: number): void {
    const binds: Nullable<Float32Array> = this.model?.skeletonBinds ?? null;

    if (!binds) {
      return;
    }

    const boneCount: number = binds.length / FLOATS_PER_BONE;
    const stride: number = boneCount * floatsPerBone;
    const base: number = frame * stride;
    const posed: Nullable<Float32Array> =
      transforms && floatsPerBone > 0 && transforms.length >= base + stride ? transforms : null;

    for (let bone: number = 0; bone < boneCount; bone += 1) {
      const source: Float32Array = posed ?? binds;
      const offset: number = posed ? base + bone * floatsPerBone : bone * FLOATS_PER_BONE;

      this.poseBone(bone, source, offset);
    }

    this.poseOverlay(posed, base, floatsPerBone);
  }

  /**
   * Marks one joint in the viewport, or clears the mark.
   *
   * @param position - Joint position in renderer space, or null to clear the mark.
   */
  public setHighlightedJoint(position: Nullable<[number, number, number]>): void {
    this.highlightedJoint = position;

    if (position && !this.highlight) {
      const geometry: BufferGeometry = new BufferGeometry();

      geometry.setAttribute("position", new BufferAttribute(new Float32Array(3), 3));

      this.highlight = new Points(
        geometry,
        new PointsMaterial({
          color: this.config.highlightColor,
          size: this.config.highlightSize,
          sizeAttenuation: false,
          depthTest: false,
          transparent: true,
        })
      );
      this.highlight.renderOrder = 2;

      this.scene.add(this.highlight);
    }

    if (position && this.highlight) {
      const attribute: BufferAttribute = this.highlight.geometry.getAttribute("position") as BufferAttribute;

      attribute.setXYZ(0, position[0], position[1], position[2]);
      attribute.needsUpdate = true;

      // The marker is a single point, so its bounding sphere is stale after a move and frustum culling would drop it.
      this.highlight.geometry.computeBoundingSphere();
    }

    this.applyHighlightVisibility();
  }

  /**
   * Shows the joint marker only when there is one to show and the overlay it belongs to is on.
   *
   * One place decides it, because two inputs govern it - the selection and the toggle - and either can change without
   * the other.
   */
  private applyHighlightVisibility(): void {
    if (this.highlight) {
      this.highlight.visible = Boolean(this.highlightedJoint) && (this.viewOptions?.isSkeletonVisible ?? false);
    }
  }

  /**
   * Put a loaded texture on one submesh.
   *
   * Applied per submesh rather than per model because a visual's children each declare their own reference and they
   * arrive one at a time, so a model shows its first texture without waiting for its last.
   *
   * A texture for a submesh this model does not have is disposed rather than kept: it belongs to a model the user has
   * already moved past, and holding it would leak the upload.
   *
   * @param submeshIndex - Index the submesh reports, which is what the backend resolved against.
   * @param texture - Uploaded texture the scene takes ownership of.
   */
  public applyTexture(submeshIndex: number, texture: Texture): void {
    const drawn: Nullable<IVisualSubmeshMesh> = this.meshes.get(submeshIndex) ?? null;

    if (!drawn) {
      texture.dispose();

      return;
    }

    // Idempotent because the caller is a react effect that re-runs whenever any texture lands, so it re-offers the ones
    // already applied. Without this the previous line would dispose the texture still in use.
    if (drawn.texture === texture) {
      return;
    }

    drawn.texture?.dispose();
    drawn.texture = texture;

    if (!this.viewOptions?.isCheckerVisible) {
      drawn.mesh.material.map = texture;
      drawn.mesh.material.needsUpdate = true;
    }
  }

  /**
   * Applies toolbar view toggles to every mesh and helper in the scene.
   *
   * @param options - Wireframe, checkerboard, grid, and axes visibility to retain for later texture arrivals.
   */
  public applyViewOptions(options: IVisualPreviewViewOptions): void {
    this.viewOptions = options;

    for (const { mesh, texture } of this.meshes.values()) {
      mesh.material.wireframe = options.isWireframe;
      mesh.material.map = options.isCheckerVisible ? this.checker : texture;
      mesh.material.needsUpdate = true;
    }

    this.grid.visible = options.isGridVisible;
    this.axes.visible = options.isAxesVisible;

    if (this.skeleton) {
      this.skeleton.visible = options.isSkeletonVisible;
    }

    this.applyHighlightVisibility();
  }

  /**
   * Frame the model from its measured extent.
   *
   * A constant distance cannot serve this viewer: loose visuals run from a pistol a few centimetres across to an actor
   * two metres tall, so resetting the camera re-fits rather than returning to a fixed point.
   */
  public resetCamera(): void {
    const { cameraFieldOfView, cameraFitMargin, cameraDirection } = this.config;

    const radius: number = this.model?.fit.radius ?? FALLBACK_RADIUS;
    const [x, y, z] = this.model?.fit.center ?? [0, 0, 0];
    const distance: number = (radius / Math.sin((cameraFieldOfView * Math.PI) / 360)) * cameraFitMargin;
    const length: number = Math.hypot(cameraDirection[0], cameraDirection[1], cameraDirection[2]);

    this.camera.position.set(
      x + (cameraDirection[0] / length) * distance,
      y + (cameraDirection[1] / length) * distance,
      z + (cameraDirection[2] / length) * distance
    );
    this.camera.near = Math.max(distance / 1000, 0.0001);
    this.camera.far = distance * 100;
    this.camera.updateProjectionMatrix();

    this.controls.target.set(x, y, z);
    this.controls.update();
  }

  /**
   * Attaches the renderer to a viewport and starts its render loop.
   *
   * @param container - Element whose dimensions drive the renderer and camera aspect ratio.
   */
  public mount(container: HTMLElement): void {
    this.container = container;
    container.appendChild(this.renderer.domElement);

    this.resizeObserver.observe(container);
    this.resize();
    this.renderFrame();
  }

  /** Stops rendering, detaches the canvas, and releases the scene's WebGL resources. */
  public dispose(): void {
    cancelAnimationFrame(this.frameHandle);

    this.resizeObserver.disconnect();
    this.controls.dispose();
    this.clearModel();

    if (this.highlight) {
      this.scene.remove(this.highlight);
      this.highlight.geometry.dispose();
      this.highlight.material.dispose();
      this.highlight = null;
    }

    this.checker.dispose();
    this.renderer.dispose();
    this.renderer.domElement.remove();

    this.container = null;
  }

  /**
   * Take the current model off the scene and free everything it owns.
   *
   * Materials and textures are per submesh now, so they are the model's to free rather than the scene's: leaving them
   * behind would leak one upload per submesh every time the user opens another visual.
   */
  private clearModel(): void {
    for (const { mesh, texture } of this.meshes.values()) {
      this.scene.remove(mesh);
      mesh.geometry.dispose();
      mesh.material.dispose();
      texture?.dispose();
    }

    this.meshes = new Map();

    if (this.skeleton) {
      this.scene.remove(this.skeleton);
      this.skeleton.geometry.dispose();
      this.skeleton.material.dispose();
      this.skeleton = null;
    }

    for (const bone of this.bones) {
      this.scene.remove(bone);
    }

    // Disposing releases the bone texture the renderer uploaded for it, which is one texture per model opened.
    this.skin?.dispose();
    this.skin = null;
    this.bones = [];

    // The marker itself survives a model change, but what it was pointing at does not. The owner re-sends the
    // selection against the replacement model, which resolves to nothing when that model has no such bone.
    this.highlightedJoint = null;
    this.applyHighlightVisibility();
  }

  /**
   * Writes one bone's transform out of a flat buffer into its matrix.
   *
   * The twelve floats are already a column-major 4x4's three basis columns and its translation, so they are written
   * straight into `elements` rather than through `Matrix4.set`, which takes its arguments row major and would silently
   * transpose them. `matrixWorldNeedsUpdate` because these bones do not derive their matrices.
   *
   * @param bone - Bone index, which is also its index in the buffer.
   * @param source - Buffer holding transforms twelve floats apart.
   * @param offset - Where this bone's twelve floats start.
   */
  private poseBone(bone: number, source: Float32Array, offset: number): void {
    const target: Optional<Bone> = this.bones[bone];

    if (!target) {
      return;
    }

    const elements: Array<number> = target.matrix.elements;

    elements[0] = source[offset];
    elements[1] = source[offset + 1];
    elements[2] = source[offset + 2];
    elements[4] = source[offset + 3];
    elements[5] = source[offset + 4];
    elements[6] = source[offset + 5];
    elements[8] = source[offset + 6];
    elements[9] = source[offset + 7];
    elements[10] = source[offset + 8];
    elements[12] = source[offset + 9];
    elements[13] = source[offset + 10];
    elements[14] = source[offset + 11];

    target.matrixWorldNeedsUpdate = true;
  }

  /**
   * Moves the overlay's segment endpoints to where the posed bones now are.
   *
   * The pairs say which two bones each drawn segment joins, and a bone's translation is the last three of its twelve
   * floats, so this reads the same buffer the matrices came from rather than being sent positions of its own.
   *
   * @param posed - Motion buffer to read, or null to fall back to the bind transforms.
   * @param base - Where the current frame starts in that buffer.
   * @param floatsPerBone - Floats one bone occupies in it.
   */
  private poseOverlay(posed: Nullable<Float32Array>, base: number, floatsPerBone: number): void {
    const pairs: Nullable<Uint16Array> = this.model?.skeletonPairs ?? null;
    const binds: Nullable<Float32Array> = this.model?.skeletonBinds ?? null;

    if (!this.skeleton || !pairs || !binds) {
      return;
    }

    const attribute: BufferAttribute = this.skeleton.geometry.getAttribute("position") as BufferAttribute;
    const stride: number = posed ? floatsPerBone : FLOATS_PER_BONE;
    const start: number = posed ? base : 0;

    for (let segment: number = 0; segment < pairs.length / 2; segment += 1) {
      const source: Float32Array = posed ?? binds;
      const child: number = start + pairs[segment * 2] * stride + TRANSLATION_OFFSET;
      const parent: number = start + pairs[segment * 2 + 1] * stride + TRANSLATION_OFFSET;

      attribute.array.set(source.subarray(child, child + 3), segment * 6);
      attribute.array.set(source.subarray(parent, parent + 3), segment * 6 + 3);
    }

    attribute.needsUpdate = true;

    this.skeleton.geometry.computeBoundingSphere();
  }

  /**
   * Builds the bone objects and the skeleton the model's skinned submeshes bind to.
   *
   * Bone inverses are taken here, once per model, from the bind transforms the backend composed: a vertex is posed by
   * its bone's current transform times the inverse of where that bone started, and inverting fourty-odd matrices once
   * is nothing next to doing it per frame.
   */
  private applySkin(model: Nullable<IVisualModelViews>): void {
    const binds: Nullable<Float32Array> = model?.skeletonBinds ?? null;

    if (!binds) {
      return;
    }

    const inverses: Array<Matrix4> = [];

    for (let bone: number = 0; bone < binds.length / FLOATS_PER_BONE; bone += 1) {
      const joint: Bone = new Bone();

      joint.matrixAutoUpdate = false;

      this.bones.push(joint);
      this.scene.add(joint);
      this.poseBone(bone, binds, bone * FLOATS_PER_BONE);

      inverses.push(joint.matrix.clone().invert());
    }

    this.skin = new Skeleton(this.bones, inverses);
  }

  /**
   * Builds the bind pose overlay, when the model came with one.
   *
   * `depthTest` off so the skeleton shows through the mesh it sits inside, which is the only way it answers where a
   * bone is. Kept out of `meshes` because it is not a submesh: no texture ever addresses it, and the detail control
   * has no range to set on it.
   */
  private applySkeleton(model: Nullable<IVisualModelViews>): void {
    const positions: Nullable<Float32Array> = model?.skeleton ?? null;

    if (!positions) {
      return;
    }

    const geometry: BufferGeometry = new BufferGeometry();

    // A copy, not the model's own array: posing writes into this attribute every frame, and the model's bind positions
    // have to survive being posed so the overlay can go back to them.
    geometry.setAttribute("position", new BufferAttribute(new Float32Array(positions), 3));

    this.skeleton = new LineSegments(
      geometry,
      new LineBasicMaterial({ color: this.config.skeletonColor, depthTest: false, transparent: true })
    );
    this.skeleton.renderOrder = 1;
    this.skeleton.visible = this.viewOptions?.isSkeletonVisible ?? false;

    this.scene.add(this.skeleton);
  }

  /** Size the helpers to the model, so the grid reads as ground rather than as a backdrop. */
  private applyScale(): void {
    const radius: number = this.model?.fit.radius ?? FALLBACK_RADIUS;

    this.grid.scale.setScalar(radius / 2);
    this.axes.scale.setScalar(radius);
  }

  /**
   * Note a size change without acting on it.
   *
   * `setSize` clears the drawing buffer, and doing that in the observer callback can paint before the frame that
   * refills it. Recording the request and applying it immediately before the next render keeps both in one frame, and
   * needs no timer: the frame loop is already the rate limit.
   */
  private resize(): void {
    this.isResizePending = true;
  }

  private applyPendingResize(): void {
    if (!this.isResizePending || !this.container) {
      return;
    }

    const width: number = this.container.clientWidth;
    const height: number = this.container.clientHeight;

    if (!width || !height) {
      return;
    }

    this.isResizePending = false;

    if (width === this.renderedWidth && height === this.renderedHeight) {
      return;
    }

    this.renderedWidth = width;
    this.renderedHeight = height;

    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(width, height);
  }

  private renderFrame(): void {
    this.frameHandle = requestAnimationFrame(() => this.renderFrame());

    this.applyPendingResize();

    this.controls.update();
    this.renderer.render(this.scene, this.camera);
  }
}
