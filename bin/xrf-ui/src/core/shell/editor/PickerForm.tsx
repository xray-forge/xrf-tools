import { default as ExpandLessIcon } from "@mui/icons-material/ExpandLess";
import { default as ExpandMoreIcon } from "@mui/icons-material/ExpandMore";
import {
  Alert,
  Box,
  Button,
  Card,
  CircularProgress,
  Divider,
  IconButton,
  Stack,
  Tooltip,
  Typography,
} from "@mui/material";
import { FormEvent, KeyboardEvent, ReactElement, ReactNode, useCallback, useEffect, useRef, useState } from "react";
import { NavigateFunction, useNavigate } from "react-router-dom";

import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { useEditorBusy } from "@/core/shell/EditorBusyContext";
import { DELAYED_REVEAL_SHORT_SX } from "@/core/ui/layout/delayed-reveal";
import { Maybe } from "@/lib/types/general";

/** Wide enough for a full windows path at the monospace size the picker rows use. */
const PANEL_WIDTH: number = 560;

/**
 * The least room a result keeps, whatever the parameters above it are doing.
 */
const RESULT_MIN_HEIGHT: number = 380;

export interface IPickerFormProps {
  title?: ReactNode;
  /** What the command reads and writes, in one line. Say it before it runs, not after. */
  description?: ReactNode;
  /** The parameter rows. */
  children?: ReactNode;
  submitLabel?: string;
  isSubmitDisabled?: boolean;
  /** Follow-up actions shown beside the primary one, such as opening an output directory. */
  secondaryActions?: ReactNode;
  isLoading?: boolean;
  error?: Maybe<ReactNode>;
  /** How the run went, for a screen that produces no result. A result says it in its own headline. */
  status?: Maybe<ReactNode>;
  result?: Maybe<ReactNode>;
  onSubmit?: () => void;
}

/**
 * Shared shell layout for the editors' "pick some paths, run a command, read the output" screens.
 */
export function PickerForm({
  title,
  description,
  children,
  submitLabel,
  isSubmitDisabled,
  onSubmit,
  secondaryActions,
  error,
  isLoading,
  status,
  result,
}: IPickerFormProps): ReactElement {
  const navigate: NavigateFunction = useNavigate();

  // Blocks navigation away from a running command, not just this form controls.
  useEditorBusy(Boolean(isLoading));

  const parametersRef = useRef<HTMLDivElement>(null);
  const [isCollapsed, setCollapsed] = useState<boolean>(false);

  const hasUserExpanded = useRef<boolean>(false);
  const hasResult: boolean = Boolean(result);

  const onToggleCollapsed = useCallback(() => {
    setCollapsed((it) => {
      if (it) {
        hasUserExpanded.current = true;
      }

      return !it;
    });
  }, []);

  const onFormSubmit = useCallback(
    (event: FormEvent) => {
      event.preventDefault();

      if (onSubmit && !isSubmitDisabled && !isLoading) {
        onSubmit();
      }
    },
    [isSubmitDisabled, isLoading, onSubmit]
  );

  const onLeave = useCallback(() => navigate("/", { replace: true }), [navigate]);

  const onFormKeyDown = useCallback(
    (event: KeyboardEvent) => {
      // Escape leaves the same way the button does, unless a command is still running.
      if (event.key === "Escape" && !isLoading) {
        onLeave();
      }
    },
    [isLoading, onLeave]
  );

  // Land on the first thing still to fill in, rather than making the user click into the form.
  useEffect(() => {
    const inputs: Array<HTMLInputElement> = Array.from(parametersRef.current?.querySelectorAll("input") ?? []);

    inputs.find((input) => !input.value)?.focus();
  }, []);

  // The form folds away when a result arrives, until the user says otherwise. Keyed on whether there
  // is a result rather than on its identity, so re-running with the form deliberately open does not
  // fold it again.
  useEffect(() => {
    if (hasResult && !hasUserExpanded.current) {
      setCollapsed(true);
    }

    if (!hasResult) {
      setCollapsed(false);
    }
  }, [hasResult]);

  return (
    <EditorLayout toolbar={<EditorToolbar />}>
      <Box
        component={"form"}
        noValidate={true}
        sx={{
          display: "flex",
          flexDirection: "column",
          width: "100%",
          height: "100%",
          minHeight: 0,
          overflowY: "auto",
        }}
        onSubmit={onFormSubmit}
        onKeyDown={onFormKeyDown}
      >
        <Box
          sx={{
            flexShrink: 0,
            padding: 3,
            paddingBottom: result ? 2 : 3,
          }}
        >
          <Card variant={"outlined"} sx={{ position: "relative", width: "100%", maxWidth: PANEL_WIDTH }}>
            <Box sx={{ display: "flex", alignItems: "flex-start", gap: 1, padding: 2 }}>
              <Box sx={{ flexGrow: 1, minWidth: 0 }}>
                {title ? (
                  <Typography component={"h1"} variant={"subtitle1"}>
                    {title}
                  </Typography>
                ) : null}

                {description ? (
                  <Typography variant={"body2"} sx={{ marginTop: 0.25, color: "text.secondary" }}>
                    {description}
                  </Typography>
                ) : null}
              </Box>

              {result ? (
                <Tooltip title={isCollapsed ? "Show parameters" : "Hide parameters"}>
                  <IconButton
                    aria-label={isCollapsed ? "Show parameters" : "Hide parameters"}
                    sx={{ flexShrink: 0 }}
                    onClick={onToggleCollapsed}
                  >
                    {isCollapsed ? <ExpandMoreIcon /> : <ExpandLessIcon />}
                  </IconButton>
                </Tooltip>
              ) : null}
            </Box>

            {isCollapsed ? null : (
              <>
                <Divider />

                <Stack ref={parametersRef} spacing={2} sx={{ padding: 2 }}>
                  {children}

                  {error ? (
                    <Alert severity={"error"} variant={"outlined"}>
                      {String(error)}
                    </Alert>
                  ) : null}
                </Stack>
              </>
            )}

            {status ? (
              <>
                <Divider />

                <Box sx={{ padding: 2 }}>{status}</Box>
              </>
            ) : null}

            <Divider />

            <Box sx={{ display: "flex", alignItems: "center", gap: 1, padding: 1.5 }}>
              <Button type={"button"} color={"inherit"} disabled={isLoading} onClick={onLeave}>
                Back
              </Button>

              <Box sx={{ flexGrow: 1 }} />

              {secondaryActions}

              {submitLabel ? (
                <Button
                  type={"submit"}
                  variant={"contained"}
                  disabled={isSubmitDisabled || isLoading}
                  startIcon={
                    isLoading ? (
                      <CircularProgress size={16} color={"inherit"} sx={DELAYED_REVEAL_SHORT_SX} />
                    ) : undefined
                  }
                >
                  {submitLabel}
                </Button>
              ) : null}
            </Box>
          </Card>
        </Box>

        {result ? (
          <Box
            sx={{
              display: "flex",
              flexDirection: "column",
              flexGrow: 1,
              minHeight: RESULT_MIN_HEIGHT,
              overflow: "hidden",
              paddingX: 3,
              marginBottom: 3,
            }}
          >
            {result}
          </Box>
        ) : null}
      </Box>
    </EditorLayout>
  );
}
