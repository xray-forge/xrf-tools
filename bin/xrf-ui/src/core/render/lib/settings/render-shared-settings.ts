import { IRendererSettings } from "@/core/render/lib/contract/renderer-settings";

/**
 * What the application sets for every viewport alike: how frames are paced, whether their passes are timed, and what
 * the features are set to. A viewer adds its own view to these.
 */
export interface IRenderSharedSettings extends Pick<IRendererSettings, "features" | "isGpuTimed" | "pacing"> {}
