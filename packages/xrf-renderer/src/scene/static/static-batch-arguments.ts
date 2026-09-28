import { IndirectStorageBufferAttribute } from "three/webgpu";

/** The arguments one phase of a batch draws by, read again whenever the batches' growth replaced them. */
export type TStaticBatchArguments = () => IndirectStorageBufferAttribute;
