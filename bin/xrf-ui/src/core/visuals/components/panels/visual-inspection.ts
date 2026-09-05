import { InjectionToken } from "@wirestate/core";

import { SelectedVisualDescription } from "@/core/bindings/types/xrf-app";
import { VisualBone } from "@/core/bindings/types/xrf-visual";
import { IVisualBumpStatus } from "@/core/visuals/lib/visual-bump";
import { IVisualTextureStatus } from "@/core/visuals/lib/visual-texture";
import { Nullable } from "@/lib/types/general";

/**
 * Marking and hiding bones, for a surface that lets a user do either.
 *
 * Separate from the visual being inspected because it is a capability rather than a fact: a viewer offers it, and a
 * surface that only shows what a model contains has nothing to offer here. Panels then read rather than pretending a
 * control is live.
 */
export interface IVisualBoneControls {
  /** Bone the viewport marks, by name, or null when none is selected. */
  highlightedBone: Nullable<string>;
  /** Bones the viewport collapses, by name. */
  hiddenBones: ReadonlySet<string>;
  /** The addon bones the open model carries, which are the ones worth a control of their own. */
  addonBones: Array<string>;
  highlightBone(name: Nullable<string>): void;
  toggleBoneVisibility(name: string): void;
  showAllBones(): void;
}

/**
 * The visual a panel is inspecting, whichever application put it on screen.
 *
 * Panels resolve this rather than an application's own service, so one header, materials, bones and motions panel
 * serve the explorer, the sequencer and whatever comes next. Everything here is observable state on the implementing
 * service, so a panel re-renders on the fields it actually reads.
 */
export interface IVisualInspection {
  /** What the backend reported about the open visual, or null when nothing is open. */
  selected: Nullable<SelectedVisualDescription>;
  /** The open model's skeleton, or no bones at all when nothing is open. */
  bones: Array<VisualBone>;
  /** What became of each submesh's texture, by submesh index. */
  textureStatuses: ReadonlyMap<number, IVisualTextureStatus>;
  /** What became of each submesh's bump pair, by submesh index, for the submeshes whose material bound one. */
  bumpStatuses: ReadonlyMap<number, IVisualBumpStatus>;
  /** Bone marking and hiding, or null on a surface that offers neither. */
  boneControls: Nullable<IVisualBoneControls>;
}

/**
 * The visual the panels of this application inspect.
 *
 * A token rather than a service class, because every application already owns a service that knows what it has open:
 * binding names which one, at the composition root where an application declares everything else about itself.
 *
 * ```ts
 * container: { bindings: [VisualsService, { token: VISUAL_INSPECTION, factory: (it) => it.get(VisualsService) }] }
 * ```
 */
export const VISUAL_INSPECTION: InjectionToken<IVisualInspection> = new InjectionToken<IVisualInspection>(
  "VISUAL_INSPECTION"
);
