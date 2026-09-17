import { Button, Typography } from "@mui/material";
import { ReactElement } from "react";
import { NavigateFunction, useNavigate } from "react-router-dom";

import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";

export function NavigationError(): ReactElement {
  const navigate: NavigateFunction = useNavigate();

  return (
    <EditorLayout toolbar={<EditorToolbar title={"Not found"} />}>
      <div className={"h-full w-full overflow-y-auto p-6"}>
        <Typography variant={"subtitle1"}>This route does not exist</Typography>

        <Typography variant={"body2"} sx={{ color: "text.secondary", marginTop: 0.5, marginBottom: 2 }}>
          The link may be out of date, or the tool it pointed at has been renamed.
        </Typography>

        <Button variant={"contained"} onClick={() => navigate("/", { replace: true })}>
          Go home
        </Button>
      </div>
    </EditorLayout>
  );
}
