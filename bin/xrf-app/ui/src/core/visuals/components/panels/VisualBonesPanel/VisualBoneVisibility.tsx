import { useInjection } from "@wirestate/react";
import { EMPTY_SET } from "@xrf/collections";
import { Nullable } from "@xrf/types";
import { EditorIconAction } from "@xrf/ui/editor/icon-action";
import { EditorPanelSection } from "@xrf/ui/editor/panel";
import { EditorPopoverNote, EditorPopoverSwitchRow } from "@xrf/ui/editor/popover-row";
import { RestartAltIcon } from "@xrf/ui/icons/restart-alt";
import { VisibilityIcon } from "@xrf/ui/icons/visibility";
import { VisibilityOffIcon } from "@xrf/ui/icons/visibility-off";
import { Chip } from "@xrf/ui/material/chip";
import { Typography } from "@xrf/ui/material/typography";
import { ReactElement } from "react";

import { IVisualInspection, VISUAL_INSPECTION } from "@/core/visuals/components/panels/visual-inspection";
import { VISIBILITY_MASK_BONES } from "@/core/visuals/lib/visual-bones";

/**
 * Turning parts of the model off, the way the engine turns off an addon that is not attached.
 */
export function VisualBoneVisibility(): Nullable<ReactElement> {
  const { bones, boneControls }: IVisualInspection = useInjection(VISUAL_INSPECTION);

  const addons: Array<string> = boneControls?.addonBones ?? [];
  const hidden: ReadonlySet<string> = boneControls?.hiddenBones ?? EMPTY_SET;
  const selected: Nullable<string> = boneControls?.highlightedBone ?? null;

  // Addon bones carry their own switch, so listing them again as chips would offer the same action twice.
  const others: Array<string> = [...hidden].filter((name: string) => !addons.includes(name));

  if (!bones.length || !boneControls) {
    return null;
  }

  return (
    <EditorPanelSection
      title={"Visibility"}
      caption={"Collapses a bone to nothing, as the engine does for an addon that is not attached"}
      actions={
        <>
          {selected ? (
            <EditorIconAction
              label={hidden.has(selected) ? `Show ${selected}` : `Hide ${selected}`}
              description={
                hidden.has(selected)
                  ? "Shows the bone selected above again"
                  : "Hides the bone selected above, and everything parented to it"
              }
              icon={hidden.has(selected) ? <VisibilityIcon /> : <VisibilityOffIcon />}
              onClick={() => boneControls.toggleBoneVisibility(selected)}
            />
          ) : null}

          {hidden.size ? (
            <EditorIconAction
              label={"Show all"}
              description={"Shows every bone hidden again"}
              icon={<RestartAltIcon />}
              onClick={boneControls.showAllBones}
            />
          ) : null}
        </>
      }
    >
      {addons.map((name: string) => (
        <EditorPopoverSwitchRow
          key={name}
          label={name}
          description={`Shows ${name}, as the game does while its addon is attached`}
          isChecked={!hidden.has(name)}
          onChange={() => boneControls.toggleBoneVisibility(name)}
        />
      ))}

      {selected ? null : (
        <EditorPopoverNote>Pick a bone above to hide it, and everything parented to it.</EditorPopoverNote>
      )}

      {others.length ? (
        <div className={"mt-2 flex flex-wrap gap-1"}>
          {others.map((name: string) => (
            <Chip key={name} size={"small"} label={name} onDelete={() => boneControls.toggleBoneVisibility(name)} />
          ))}
        </div>
      ) : null}

      {bones.length > VISIBILITY_MASK_BONES ? (
        <Typography className={"mt-2 block text-warning"} variant={"caption"}>
          {`${bones.length - VISIBILITY_MASK_BONES} of these bones sit past the engine's ` +
            `${VISIBILITY_MASK_BONES} bone visibility mask, so hiding one of them is a viewer state the engine ` +
            `cannot reach.`}
        </Typography>
      ) : null}
    </EditorPanelSection>
  );
}
