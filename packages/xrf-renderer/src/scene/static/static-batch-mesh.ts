import { LineSegments, Mesh } from "three/webgpu";

/** What a batch draws a phase with: a mesh of its clusters' triangles, or line segments of their edges. */
export type TStaticBatchMesh = Mesh | LineSegments;
