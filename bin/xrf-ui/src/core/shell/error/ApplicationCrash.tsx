import { default as ExpandMoreIcon } from "@mui/icons-material/ExpandMore";
import { Accordion, AccordionDetails, AccordionSummary, Button, Typography } from "@mui/material";
import { ReactElement, useCallback } from "react";
import { NavigateFunction, useNavigate } from "react-router-dom";

import { IErrorBoundaryFallbackProps } from "@/core/error/components/ErrorBoundary";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";

/**
 * What the application shell shows in place of a tool that failed to render.
 */
export function ApplicationCrash({ error, onRetry }: IErrorBoundaryFallbackProps): ReactElement {
  const navigate: NavigateFunction = useNavigate();

  const onGoHome = useCallback(() => {
    navigate("/", { replace: true });
  }, [navigate]);

  const onReload = useCallback(() => {
    window.location.reload();
  }, []);

  return (
    <EditorLayout toolbar={<EditorToolbar title={"Something went wrong"} />}>
      <div className={"h-full w-full overflow-y-auto p-6"}>
        <Typography variant={"subtitle1"}>This tool stopped rendering</Typography>

        <Typography className={"mt-1 mb-4 text-text-secondary"} variant={"body2"}>
          The rest of the application is still running. Try again to re-render it, or switch to another tool from the
          rail.
        </Typography>

        <div className={"mb-6 flex flex-row gap-2"}>
          <Button variant={"contained"} onClick={onRetry}>
            Try again
          </Button>

          <Button variant={"outlined"} onClick={onGoHome}>
            Go home
          </Button>

          <Button color={"inherit"} onClick={onReload}>
            Reload window
          </Button>
        </div>

        <Accordion disableGutters variant={"outlined"}>
          <AccordionSummary expandIcon={<ExpandMoreIcon fontSize={"small"} />}>
            <Typography variant={"body2"}>Details</Typography>
          </AccordionSummary>

          <AccordionDetails>
            <Typography
              className={"monospace m-0 wrap-break-word whitespace-pre-wrap text-text-secondary"}
              component={"pre"}
              variant={"caption"}
            >
              {error.stack ?? String(error)}
            </Typography>
          </AccordionDetails>
        </Accordion>
      </div>
    </EditorLayout>
  );
}
