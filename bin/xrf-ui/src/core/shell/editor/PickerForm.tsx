import { default as ExpandLessIcon } from "@mui/icons-material/ExpandLess";
import { default as ExpandMoreIcon } from "@mui/icons-material/ExpandMore";
import { Alert, Button, Card, CircularProgress, IconButton, Tooltip, Typography } from "@mui/material";
import { FormEvent, KeyboardEvent, ReactElement, ReactNode, useCallback, useEffect, useRef, useState } from "react";
import { NavigateFunction, useNavigate } from "react-router-dom";

import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { useEditorBusy, useRequestLeave } from "@/core/shell/editor-lifecycle";
import { FORM_SURFACE_SX } from "@/core/theme/form-surface";
import { FormCommitContext, IFormCommitRegistry, useFormCommitRegistry } from "@/core/ui/form/form-commit";
import { DELAYED_REVEAL_SHORT_SX } from "@/core/ui/layout/delayed-reveal";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Logger, useLogger } from "@/lib/logging";
import { Maybe } from "@/lib/types/general";

interface IPickerFormProps extends BaseComponentProps {
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
  "data-testid": dataTestId,
  id,
  className,
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
  const log: Logger = useLogger(__MODULE_NAME__);
  const navigate: NavigateFunction = useNavigate();

  // Submission is the only moment this shell knows a parameter was meant rather than merely typed, and the rows are
  // the ones that know which fields they hold.
  const fields: IFormCommitRegistry = useFormCommitRegistry();

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
      log.info("Submit form:", { fields });

      event.preventDefault();

      if (onSubmit && !isSubmitDisabled && !isLoading) {
        fields.commit();
        onSubmit();
      }
    },
    [log, onSubmit, isSubmitDisabled, isLoading, fields]
  );

  const requestLeave = useRequestLeave();

  const onLeave = useCallback(() => {
    log.info("Requesting leave");

    requestLeave(() => navigate("/", { replace: true }));
  }, [log, navigate, requestLeave]);

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

  // Blocks navigation away from a running command, not just this form controls.
  useEditorBusy(Boolean(isLoading));

  return (
    <EditorLayout data-testid={dataTestId} id={id} className={className} toolbar={<EditorToolbar />}>
      <form
        noValidate={true}
        className={"flex h-full min-h-0 w-full flex-col overflow-y-auto"}
        onSubmit={onFormSubmit}
        onKeyDown={onFormKeyDown}
      >
        <div className={result ? "flex shrink-0 justify-center p-6 pb-4" : "flex shrink-0 justify-center p-6"}>
          <Card
            className={"relative flex w-full max-w-reading flex-col gap-6 p-6"}
            sx={FORM_SURFACE_SX}
            variant={"elevation"}
            elevation={0}
          >
            <div className={"flex items-start gap-2"}>
              <div className={"min-w-0 grow"}>
                {title ? (
                  <Typography component={"h1"} variant={"subtitle1"}>
                    {title}
                  </Typography>
                ) : null}

                {description ? (
                  <Typography className={"mt-0.5 text-text-secondary"} variant={"body2"}>
                    {description}
                  </Typography>
                ) : null}
              </div>

              {result ? (
                <Tooltip title={isCollapsed ? "Show parameters" : "Hide parameters"}>
                  <IconButton
                    aria-label={isCollapsed ? "Show parameters" : "Hide parameters"}
                    className={"shrink-0"}
                    onClick={onToggleCollapsed}
                  >
                    {isCollapsed ? <ExpandMoreIcon /> : <ExpandLessIcon />}
                  </IconButton>
                </Tooltip>
              ) : null}
            </div>

            {isCollapsed ? null : (
              <div ref={parametersRef} className={"flex flex-col gap-6"}>
                <FormCommitContext.Provider value={fields}>{children}</FormCommitContext.Provider>

                {error ? (
                  <Alert severity={"error"} variant={"outlined"}>
                    {String(error)}
                  </Alert>
                ) : null}
              </div>
            )}

            {status ? <div>{status}</div> : null}

            <div className={"flex items-center gap-2"}>
              <Button type={"button"} color={"inherit"} disabled={isLoading} onClick={onLeave}>
                Back
              </Button>

              <div className={"grow"} />

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
            </div>
          </Card>
        </div>

        {result ? <div className={"mb-6 flex min-h-95 grow flex-col overflow-hidden px-6"}>{result}</div> : null}
      </form>
    </EditorLayout>
  );
}
