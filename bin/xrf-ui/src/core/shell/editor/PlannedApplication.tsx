import { default as ConstructionIcon } from "@mui/icons-material/Construction";
import { ReactElement } from "react";

import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { getApplicationBackgroundSx } from "@/core/theme/application-background";
import { EmptyState } from "@/core/ui/layout/EmptyState";

interface IPlannedApplicationProps {
  /** What this application will do once it exists, in the present tense. */
  description: string;
}

/**
 * What an application on the roadmap shows before it is built.
 */
export function PlannedApplication({ description }: IPlannedApplicationProps): ReactElement {
  return (
    <EditorLayout toolbar={<EditorToolbar />} sx={getApplicationBackgroundSx}>
      <EmptyState
        icon={<ConstructionIcon sx={{ fontSize: 40, color: "text.secondary", opacity: 0.55 }} />}
        title={"Not implemented yet"}
        description={description}
      />
    </EditorLayout>
  );
}
