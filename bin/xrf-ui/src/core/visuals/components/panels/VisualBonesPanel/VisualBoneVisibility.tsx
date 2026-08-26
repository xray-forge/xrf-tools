import { default as VisibilityIcon } from "@mui/icons-material/Visibility";
import { default as VisibilityOffIcon } from "@mui/icons-material/VisibilityOff";
import { Box, Button, Chip, FormControlLabel, Switch, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { IVisualInspection, VISUAL_INSPECTION } from "@/core/visuals/components/panels/visual-inspection";
import { VisualPanelSection } from "@/core/visuals/components/panels/VisualPanelSection";
import { VISIBILITY_MASK_BONES } from "@/core/visuals/lib/visual-bones";
import { Nullable } from "@/lib/types/general";

/** Stable, so a surface offering no bone controls does not hand the section a new set on every render. */
const EMPTY_HIDDEN: ReadonlySet<string> = new Set();

/**
 * Turning parts of the model off, the way the engine turns off an addon that is not attached.
 *
 * The addon bones get controls of their own because they are the ones anyone looks for. Anything else is hidden through
 * the tree above: select a bone, then hide it.
 */
export function VisualBoneVisibility(): Nullable<ReactElement> {
  const { bones, boneControls }: IVisualInspection = useInjection(VISUAL_INSPECTION);

  const addons: Array<string> = boneControls?.addonBones ?? [];
  const hidden: ReadonlySet<string> = boneControls?.hiddenBones ?? EMPTY_HIDDEN;
  const selected: Nullable<string> = boneControls?.highlightedBone ?? null;

  // Addon bones carry their own switch, so listing them again as chips would offer the same action twice.
  const others: Array<string> = [...hidden].filter((name: string) => !addons.includes(name));

  if (!bones.length || !boneControls) {
    return null;
  }

  return (
    <VisualPanelSection
      title={"Visibility"}
      caption={"Collapses a bone to nothing, as the engine does for an addon that is not attached"}
    >
      {addons.map((name: string) => (
        <FormControlLabel
          key={name}
          sx={{ display: "flex", marginLeft: 0, justifyContent: "space-between" }}
          labelPlacement={"start"}
          label={<Typography variant={"body2"}>{name}</Typography>}
          control={
            <Switch
              size={"small"}
              checked={!hidden.has(name)}
              slotProps={{ input: { "aria-label": `Show ${name}` } }}
              onChange={() => boneControls.toggleBoneVisibility(name)}
            />
          }
        />
      ))}

      <Box sx={{ display: "flex", alignItems: "center", gap: 1, flexWrap: "wrap", marginTop: addons.length ? 1 : 0 }}>
        {selected ? (
          <Button
            size={"small"}
            startIcon={hidden.has(selected) ? <VisibilityIcon /> : <VisibilityOffIcon />}
            onClick={() => boneControls.toggleBoneVisibility(selected)}
          >
            {hidden.has(selected) ? `Show ${selected}` : `Hide ${selected}`}
          </Button>
        ) : (
          <Typography variant={"caption"} sx={{ color: "text.disabled" }}>
            Pick a bone above to hide it, and everything parented to it.
          </Typography>
        )}

        {hidden.size ? (
          <Button size={"small"} onClick={boneControls.showAllBones}>
            Show all
          </Button>
        ) : null}
      </Box>

      {others.length ? (
        <Box sx={{ display: "flex", gap: 0.5, flexWrap: "wrap", marginTop: 1 }}>
          {others.map((name: string) => (
            <Chip key={name} size={"small"} label={name} onDelete={() => boneControls.toggleBoneVisibility(name)} />
          ))}
        </Box>
      ) : null}

      {bones.length > VISIBILITY_MASK_BONES ? (
        <Typography variant={"caption"} sx={{ display: "block", marginTop: 1, color: "warning.main" }}>
          {`${bones.length - VISIBILITY_MASK_BONES} of these bones sit past the engine's ` +
            `${VISIBILITY_MASK_BONES} bone visibility mask, so hiding one of them is a viewer state the engine ` +
            `cannot reach.`}
        </Typography>
      ) : null}
    </VisualPanelSection>
  );
}
