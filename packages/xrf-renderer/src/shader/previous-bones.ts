import { Skeleton } from "three/webgpu";

/** The property a skeleton keeps the bone matrices of the frame before under, which the skinned motion reads. */
export const PREVIOUS_BONE_MATRICES = "previousBoneMatrices";

/** A skeleton keeping the bone matrices of the frame before. */
export type TPreviousSkeleton = Skeleton & { [PREVIOUS_BONE_MATRICES]: Float32Array };
