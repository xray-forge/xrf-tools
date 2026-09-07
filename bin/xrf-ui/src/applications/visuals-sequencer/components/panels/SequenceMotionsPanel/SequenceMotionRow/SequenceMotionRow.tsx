import { default as AddIcon } from "@mui/icons-material/Add";
import { Box, Typography } from "@mui/material";
import { ReactElement } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface ISequenceMotionRowProps extends BaseComponentProps {
  motion: string;
  usageCount: number;
  onAdd: (motion: string) => void;
}

/** One available motion, its use in the track, and the action to add another clip. */
export function SequenceMotionRow({
  "data-testid": dataTestId = "sequence-motion-row",
  id,
  className,
  motion,
  usageCount,
  onAdd,
}: ISequenceMotionRowProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ display: "flex", alignItems: "center", gap: 1, paddingLeft: 1, paddingY: 0.2, lineHeight: 1.6 }}
    >
      <Typography variant={"body2"} sx={{ flexGrow: 1, wordBreak: "break-all" }}>
        {motion}
      </Typography>

      {usageCount > 0 ? (
        <Typography variant={"caption"} sx={{ color: "text.disabled", flexShrink: 0 }}>
          {`×${usageCount}`}
        </Typography>
      ) : null}

      <EditorIconAction
        label={`Add ${motion}`}
        description={"Add to the track"}
        icon={<AddIcon />}
        onClick={() => onAdd(motion)}
      />
    </Box>
  );
}
