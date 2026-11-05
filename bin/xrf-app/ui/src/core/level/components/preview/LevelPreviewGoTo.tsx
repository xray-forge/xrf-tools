import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { InputBase } from "@xrf/ui/material/input";
import { cn } from "@xrf/ui/theme";
import { ChangeEvent, FocusEvent, FormEvent, KeyboardEvent, ReactElement, useCallback, useState } from "react";

import { formatLevelGoTo, ILevelGoTo, parseLevelGoTo, toLevelGoTo } from "@/core/level/lib/camera/level-camera-goto";
import { LevelRenderService } from "@/core/level/services/level-render";
import { LevelViewportService } from "@/core/level/services/level-viewport";

/** What the field says of text stating no place, and what it takes instead. */
const UNREAD: string = "Names no place: x 0 y 0 z 0 h 0 p 0, part of it, or numbers in that order";

/**
 * The camera readout turned into a field, in its corner and its type: where the camera stood as it opened, which a
 * person edits or pastes over, `Enter` going there and `Escape` or a click away leaving it. A value the text leaves out
 * stays as it was.
 */
export function LevelPreviewGoTo({
  "data-testid": dataTestId = "level-preview-go-to",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewportService: LevelViewportService = useInjection(LevelViewportService);
  const renderService: LevelRenderService = useInjection(LevelRenderService);
  // Where the camera stood as the field opened: what it starts at, and what a value the text leaves out keeps.
  const [start] = useState<Nullable<ILevelGoTo>>(() =>
    viewportService.camera ? toLevelGoTo(viewportService.camera) : null
  );
  const [text, setText] = useState<string>(() => (start ? formatLevelGoTo(start) : ""));
  const parsed: Nullable<ILevelGoTo> = start ? parseLevelGoTo(text, start) : null;

  const onChange = useCallback((event: ChangeEvent<HTMLInputElement>) => setText(event.target.value), []);

  const onSubmit = useCallback(
    (event: FormEvent) => {
      event.preventDefault();

      if (parsed) {
        renderService.camera.goTo(parsed);
        viewportService.closeGoTo();
      }
    },
    [parsed, renderService, viewportService]
  );

  // `Escape` leaves the field alone, rather than reaching the scene as the key that drops the selection.
  const onKeyDown = useCallback(
    (event: KeyboardEvent<HTMLInputElement>) => {
      if (event.key === "Escape") {
        event.stopPropagation();
        viewportService.closeGoTo();
      }
    },
    [viewportService]
  );

  const onFocus = useCallback((event: FocusEvent<HTMLInputElement>) => event.currentTarget.select(), []);

  return (
    <form
      data-testid={dataTestId}
      id={id}
      className={cn(
        "absolute right-2 bottom-2 z-20 w-80 rounded-surface bg-viewport-backdrop/90 px-2 py-1",
        "font-mono text-xs leading-tight text-viewport-text",
        className
      )}
      aria-label={"Go to coordinates"}
      onSubmit={onSubmit}
    >
      <InputBase
        className={"w-full font-mono text-xs text-viewport-text"}
        autoFocus
        value={text}
        inputProps={{ "aria-label": "Coordinates", spellCheck: false, onFocus }}
        onChange={onChange}
        onKeyDown={onKeyDown}
        onBlur={viewportService.closeGoTo}
      />

      <div className={parsed ? "text-viewport-text/70" : "text-warning"} aria-live={"polite"}>
        {parsed ? `Enter goes to ${formatLevelGoTo(parsed)}` : UNREAD}
      </div>
    </form>
  );
}
