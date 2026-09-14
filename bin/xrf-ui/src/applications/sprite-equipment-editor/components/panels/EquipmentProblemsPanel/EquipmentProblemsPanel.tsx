import { Box, List, ListItem, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SpriteEquipmentEditorService } from "@/applications/sprite-equipment-editor/services/editor";
import { EquipmentGridService } from "@/applications/sprite-equipment-editor/services/grid";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { IEquipmentProblem, toEquipmentProblems } from "./equipment-problems";

/**
 * Everything worth saying about the open sheet that is not an occupant.
 */
export function EquipmentProblemsPanel({
  "data-testid": dataTestId = "equipment-problems-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const spriteEquipmentService: SpriteEquipmentEditorService = useInjection(SpriteEquipmentEditorService);
  const gridService: EquipmentGridService = useInjection(EquipmentGridService);

  const problems: Array<IEquipmentProblem> = toEquipmentProblems(
    spriteEquipmentService.spriteImage.value?.metadata ?? null,
    gridService.layout
  );

  if (!problems.length) {
    return (
      <Box data-testid={dataTestId} id={id} className={className} sx={{ padding: 1 }}>
        <Typography variant={"caption"} color={"text.secondary"}>
          Nothing to report
        </Typography>
      </Box>
    );
  }

  return (
    <List data-testid={dataTestId} id={id} className={className} dense={true} sx={{ overflow: "auto" }}>
      {problems.map((problem: IEquipmentProblem) => (
        <ListItem key={problem.kind} sx={{ display: "block", paddingY: 1 }}>
          <Typography variant={"body2"}>{problem.title}</Typography>

          <Typography variant={"caption"} color={"text.secondary"} sx={{ wordBreak: "break-word" }}>
            {problem.detail}
          </Typography>
        </ListItem>
      ))}
    </List>
  );
}
