import { describe, expect, it } from "@jest/globals";
import { Bone, LineSegments, Matrix4, Object3D } from "three";

import { VisualPreviewSkeleton } from "@/core/visuals/components/scene/VisualPreviewSkeleton";
import { FLOATS_PER_BONE, IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { mockVisualBoneFloats, mockVisualModelViews } from "@/fixtures/mocks/visual.mocks";
import { Nullable } from "@/lib/types/general";

const CONFIG = { skeletonColor: 0x00ff00 };

/** Two bones, the second a child of the first, each parked at a distinguishable translation. */
const BONE_COUNT: number = 2;

/**
 * A model carrying a two bone skeleton and one drawn segment joining them.
 *
 * The scenario every test here reads against: bone 0 sits at 1, bone 1 at 2, so a posed translation says which bone
 * produced it without any test having to spell the layout out again.
 *
 * @param overrides - Views to replace, for a model whose skeleton is partial.
 * @returns Model views carrying a skeleton.
 */
function mockSkeletalModel(overrides: Partial<IVisualModelViews> = {}): IVisualModelViews {
  return mockVisualModelViews({
    skeleton: new Float32Array([1, 1, 1, 2, 2, 2]),
    skeletonPairs: new Uint16Array([0, 1]),
    skeletonBinds: new Float32Array([...mockVisualBoneFloats(1), ...mockVisualBoneFloats(2)]),
    ...overrides,
  });
}

/**
 * A motion buffer whose every frame parks both bones at the frame's own marker.
 *
 * @param frames - How many frames to bake.
 * @returns Frame major transforms of the shape `mockSkeletalModel` expects.
 */
function mockMotion(frames: number): Float32Array {
  const transforms: Float32Array = new Float32Array(frames * BONE_COUNT * FLOATS_PER_BONE);

  for (let frame: number = 0; frame < frames; frame += 1) {
    transforms.set(
      [...mockVisualBoneFloats(10 + frame), ...mockVisualBoneFloats(20 + frame)],
      frame * BONE_COUNT * FLOATS_PER_BONE
    );
  }

  return transforms;
}

/**
 * Builds a skeleton attached to a bare node, which is all it needs from a scene.
 *
 * @param model - Model to build from.
 * @returns The skeleton and the node it attached to.
 */
function mockSkeleton(model: IVisualModelViews = mockSkeletalModel()): {
  skeleton: VisualPreviewSkeleton;
  parent: Object3D;
} {
  const parent: Object3D = new Object3D();
  const skeleton: Nullable<VisualPreviewSkeleton> = VisualPreviewSkeleton.create(model, parent, CONFIG);

  if (!skeleton) {
    throw new Error("This model was built with bind transforms and should have produced a skeleton");
  }

  return { parent, skeleton };
}

/**
 * @param parent - Node the skeleton attached to.
 * @returns Every bone attached, in the order they were added.
 */
function attachedBones(parent: Object3D): Array<Bone> {
  return parent.children.filter((it: Object3D): it is Bone => it instanceof Bone);
}

/**
 * @param parent - Node the skeleton attached to.
 * @returns The overlay's segment endpoints, or null when it drew none.
 */
function overlayPositions(parent: Object3D): Nullable<Array<number>> {
  const overlay: Nullable<Object3D> = parent.children.find((it: Object3D) => it instanceof LineSegments) ?? null;

  return overlay ? [...(overlay as LineSegments).geometry.getAttribute("position").array] : null;
}

/**
 * @param bone - Bone to read.
 * @returns The translation its matrix carries.
 */
function translationOf(bone: Bone): [number, number, number] {
  const { elements } = bone.matrix;

  return [elements[12], elements[13], elements[14]];
}

describe("VisualPreviewSkeleton", () => {
  it("reports no skeleton for a model that carries no bind data", () => {
    expect(VisualPreviewSkeleton.create(mockSkeletalModel({ skeletonBinds: null }), new Object3D(), CONFIG)).toBeNull();
  });

  it("attaches a bone per bind transform and poses each at its bind translation", () => {
    const { parent } = mockSkeleton();
    const bones: Array<Bone> = attachedBones(parent);

    expect(bones).toHaveLength(BONE_COUNT);
    expect(translationOf(bones[0])).toEqual([1, 1, 1]);
    expect(translationOf(bones[1])).toEqual([2, 2, 2]);
  });

  it("binds the skin to inverses of the bind pose, so the bind pose skins to itself", () => {
    const { skeleton, parent } = mockSkeleton();
    const bones: Array<Bone> = attachedBones(parent);

    // Bone times its own inverse is the identity: a mesh in its bind pose has to come out where it was stored.
    const posed: Matrix4 = bones[1].matrix.clone().multiply(skeleton.getSkin().boneInverses[1]);
    const identity: Array<number> = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];

    expect(posed.elements.map((it: number) => Math.round(it))).toEqual(identity);
  });

  it("poses every bone from the frame it is given", () => {
    const { skeleton, parent } = mockSkeleton();

    skeleton.setPose(mockMotion(3), 2, FLOATS_PER_BONE);

    const bones: Array<Bone> = attachedBones(parent);

    expect(translationOf(bones[0])).toEqual([12, 12, 12]);
    expect(translationOf(bones[1])).toEqual([22, 22, 22]);
  });

  it("moves the overlay's endpoints with the bones it joins", () => {
    const { skeleton, parent } = mockSkeleton();

    expect(overlayPositions(parent)).toEqual([1, 1, 1, 2, 2, 2]);

    skeleton.setPose(mockMotion(3), 1, FLOATS_PER_BONE);

    // The pair joins bone 0 to bone 1, so the segment has to follow both out of the bind pose.
    expect(overlayPositions(parent)).toEqual([11, 11, 11, 21, 21, 21]);
  });

  it("returns to the bind pose rather than indexing a buffer too short for the frame asked for", () => {
    const { skeleton, parent } = mockSkeleton();

    // A motion baked against another skeleton: posing from whatever sits at that offset would show a mangled model.
    skeleton.setPose(mockMotion(2), 5, FLOATS_PER_BONE);

    expect(translationOf(attachedBones(parent)[0])).toEqual([1, 1, 1]);
    expect(overlayPositions(parent)).toEqual([1, 1, 1, 2, 2, 2]);
  });

  it("returns to the bind pose when the motion is cleared", () => {
    const { skeleton, parent } = mockSkeleton();

    skeleton.setPose(mockMotion(3), 2, FLOATS_PER_BONE);
    skeleton.setPose(null, 0, 0);

    expect(translationOf(attachedBones(parent)[0])).toEqual([1, 1, 1]);
  });

  it("collapses a hidden bone the way the engine does, leaving the rest posed", () => {
    const { skeleton, parent } = mockSkeleton();

    skeleton.setHiddenBones(new Set([1]));

    const bones: Array<Bone> = attachedBones(parent);

    // `scale(0, 0, 0)`: an identity with its diagonal zeroed, which degenerates every triangle weighted to the bone.
    expect([...bones[1].matrix.elements]).toEqual([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
    expect(translationOf(bones[0])).toEqual([1, 1, 1]);
  });

  it("keeps a hidden bone collapsed through every frame of a motion", () => {
    const { skeleton, parent } = mockSkeleton();

    skeleton.setHiddenBones(new Set([1]));
    skeleton.setPose(mockMotion(3), 1, FLOATS_PER_BONE);

    const bones: Array<Bone> = attachedBones(parent);

    expect(translationOf(bones[1])).toEqual([0, 0, 0]);
    expect(translationOf(bones[0])).toEqual([11, 11, 11]);
  });

  it("keeps the overlay drawing a hidden bone, which is what says where the part was", () => {
    const { skeleton, parent } = mockSkeleton();

    skeleton.setHiddenBones(new Set([0, 1]));

    expect(overlayPositions(parent)).toEqual([1, 1, 1, 2, 2, 2]);
  });

  it("brings a bone back when it stops being hidden", () => {
    const { skeleton, parent } = mockSkeleton();

    skeleton.setHiddenBones(new Set([1]));
    skeleton.setHiddenBones(new Set());

    expect(translationOf(attachedBones(parent)[1])).toEqual([2, 2, 2]);
  });

  it("draws no overlay for a skeleton whose bones form no segment", () => {
    const { skeleton, parent } = mockSkeleton(mockSkeletalModel({ skeleton: null, skeletonPairs: null }));

    skeleton.setPose(mockMotion(3), 1, FLOATS_PER_BONE);

    expect(overlayPositions(parent)).toBeNull();
    expect(attachedBones(parent)).toHaveLength(BONE_COUNT);
  });

  it("detaches everything it attached", () => {
    const { skeleton, parent } = mockSkeleton();

    expect(parent.children).toHaveLength(BONE_COUNT + 1);

    skeleton.dispose();

    expect(parent.children).toHaveLength(0);
  });
});
