import { WorldMusicCommand, WorldSoundMusicReport } from "@xrf/ipc/types/xrf-world";
import { Nullable } from "@xrf/types";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { MenuItem } from "@xrf/ui/material/menu";
import { Slider } from "@xrf/ui/material/slider";
import { TextField } from "@xrf/ui/material/text-field";
import { Typography } from "@xrf/ui/material/typography";
import { CONTROL } from "@xrf/ui/theme";
import { ChangeEvent, ReactElement, RefObject, useCallback, useEffect, useRef, useState } from "react";

import { formatLevelSoundSeconds, formatLevelSoundTrackTime } from "@/tools/level-viewer/lib/sound/level-sound-format";

/** What the track list holds for the game's own pick. */
const BY_SCHEDULE: string = "";

/** A second, the seek slider's step. */
const SECOND_MS: number = 1000;

interface ILevelSoundMusicSectionProps extends BaseComponentProps {
  /** Where the level's music stands, null for a level naming none. */
  music: Nullable<WorldSoundMusicReport>;
  /** Asks the world, settling once it has answered. */
  onCommand: (command: WorldMusicCommand) => Promise<unknown>;
}

/** How far from a seek's point a report may stand and still be one made after it. */
const SEEK_REACHED_MS: number = 2000;

/** Reports after the world answered a seek past which the slider follows them whatever they say. */
const SEEK_REPORTS_MOST: number = 2;

/** A seek let go of: the point shown until a report made after the world took it replaces it. */
interface ILevelSoundSeek {
  position: number;
  track: Nullable<string>;
  /** Once the world answered: the report shown then, and how many came since. */
  answered: Nullable<{ last: Nullable<WorldSoundMusicReport>; reports: number }>;
}

/**
 * The level's music: the track playing and how far into it, which the slider plays from any point once let go, or the
 * wait for the next; and which plays, the game's schedule or one track chosen by hand.
 */
export function LevelSoundMusicSection({
  "data-testid": dataTestId = "level-sound-music-section",
  id,
  className,
  music,
  onCommand,
}: ILevelSoundMusicSectionProps): ReactElement {
  // Where the slider is held while dragged, shown and kept for its letting go, which a key's step sends in the same
  // event as its change, before the state is rendered; the music plays from there only once it is let go.
  const [dragged, setDragged] = useState<Nullable<number>>(null);
  const held: RefObject<Nullable<number>> = useRef<Nullable<number>>(null);
  // The reports come every half second, one asked before the seek perhaps answered after it: until one stands at the
  // point, plays another track, or a couple have come, the slider keeps the point rather than jump back.
  const [seek, setSeek] = useState<Nullable<ILevelSoundSeek>>(null);
  const shown: RefObject<Nullable<WorldSoundMusicReport>> = useRef<Nullable<WorldSoundMusicReport>>(music);

  const onTrack = useCallback(
    (event: ChangeEvent<HTMLInputElement>) =>
      void onCommand(
        event.target.value === BY_SCHEDULE ? { kind: "schedule" } : { kind: "play", track: event.target.value }
      ),
    [onCommand]
  );

  const onDrag = useCallback((_: Event, value: number | ReadonlyArray<number>) => {
    held.current = typeof value === "number" ? value : (value[0] ?? 0);
    setDragged(held.current);
  }, []);

  const onRelease = useCallback(() => {
    if (held.current !== null) {
      const position: number = Math.round(held.current);

      setSeek({ position, track: shown.current?.track ?? null, answered: null });
      // Only the seek still waiting on this answer takes it: a later one has replaced it.
      void onCommand({ kind: "seek", position }).finally(() =>
        setSeek((current) =>
          current?.position === position && current.answered === null
            ? { ...current, answered: { last: shown.current, reports: 0 } }
            : current
        )
      );
    }

    held.current = null;
    setDragged(null);
  }, [onCommand]);

  useEffect(() => {
    shown.current = music;
  }, [music]);

  useEffect(() => {
    if (!seek?.answered || music === seek.answered.last) {
      return;
    }

    const reports: number = seek.answered.reports + 1;
    const isReached: boolean =
      !music || music.track !== seek.track || Math.abs(music.position - seek.position) < SEEK_REACHED_MS;

    setSeek(isReached || reports >= SEEK_REPORTS_MOST ? null : { ...seek, answered: { last: music, reports } });
  }, [music, seek]);

  if (!music || music.tracks.length === 0) {
    return (
      <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Music"}>
        <Typography className={"block text-text-secondary"} variant={"caption"}>
          The level names no music.
        </Typography>
      </EditorPanelSection>
    );
  }

  const position: number = dragged ?? seek?.position ?? music.position;

  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Music"}>
      <TextField
        select
        fullWidth
        size={"small"}
        label={"Track"}
        value={music.chosen ?? BY_SCHEDULE}
        slotProps={{
          inputLabel: { shrink: true },
          select: {
            displayEmpty: true,
            MenuProps: { slotProps: { paper: { sx: { maxHeight: CONTROL.selectMenuMaxHeight } } } },
          },
        }}
        onChange={onTrack}
      >
        <MenuItem dense value={BY_SCHEDULE}>
          By schedule
        </MenuItem>
        {music.tracks.map((track: string) => (
          <MenuItem key={track} dense value={track}>
            {track}
          </MenuItem>
        ))}
      </TextField>

      {music.track ? (
        <>
          <EditorPanelProperty
            label={music.track}
            value={`${formatLevelSoundTrackTime(position)} / ${formatLevelSoundTrackTime(music.length)}`}
          />
          <Slider
            size={"small"}
            min={0}
            max={Math.max(music.length, SECOND_MS)}
            step={SECOND_MS}
            value={Math.min(position, music.length)}
            aria-label={"Play the track from"}
            getAriaValueText={formatLevelSoundTrackTime}
            onChange={onDrag}
            onChangeCommitted={onRelease}
          />
        </>
      ) : (
        <EditorPanelProperty label={"Next"} value={`In ${formatLevelSoundSeconds(music.wait)}`} />
      )}
    </EditorPanelSection>
  );
}
