import { Box, Slider, Typography } from "@mui/material";
import { ReactElement } from "react";

import { MINIMUM_GLOSS_POWER } from "@/applications/textures-editor/lib/texture-bump-gloss";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ITextureGlossFieldProps extends BaseComponentProps {
  value: number;
  isDisabled?: boolean;
  onChange: (value: number) => void;
}

/**
 * One gloss level for the whole surface, for a texture authored without a mask.
 *
 * The warning below the SDK's own threshold is shown before the run rather than after it, because it is knowable from
 * this number alone: a constant gloss is its own mean. A mask has to be read before anyone can say the same, which is
 * why that case is reported in the outcome instead.
 */
export function TextureGlossField({
  "data-testid": dataTestId = "texture-gloss-field",
  id,
  className,
  value,
  isDisabled = false,
  onChange,
}: ITextureGlossFieldProps): ReactElement {
  return (
    <Box data-testid={dataTestId} id={id} className={className} sx={{ px: 1 }}>
      <Typography variant={"caption"} color={"text.secondary"}>
        {`Gloss level ${value.toFixed(2)}`}
      </Typography>

      <Slider
        size={"small"}
        min={0}
        max={1}
        step={0.05}
        value={value}
        disabled={isDisabled}
        valueLabelDisplay={"auto"}
        onChange={(_, next: number | Array<number>) => onChange(Array.isArray(next) ? (next[0] ?? 0) : next)}
      />

      {value < MINIMUM_GLOSS_POWER ? (
        <Typography variant={"caption"} color={"warning.main"}>
          {`Below ${MINIMUM_GLOSS_POWER}, the surface shows almost no specular response. The pair is still written.`}
        </Typography>
      ) : null}
    </Box>
  );
}
