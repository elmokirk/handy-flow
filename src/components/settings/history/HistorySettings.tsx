import React, { useCallback, useEffect, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Check,
  Copy,
  FolderOpen,
  RotateCcw,
  Star,
  Trash2,
  Upload,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import {
  commands,
  type CanonicalHistoryEntry,
  type CanonicalSeamDetail,
} from "@/bindings";
import { formatDateMs, formatDateTimeMs } from "@/utils/dateFormat";
import { AudioPlayer, AudioPlayerGroup } from "../../ui/AudioPlayer";
import { Button } from "../../ui/Button";

const PAGE_SIZE = 30;
type Range = "all" | "today" | "week" | "days7" | "days30" | "custom";
type Origin = "all" | "handy" | "wispr" | "manual";
type DateBounds = { fromMs: number | null; toMs: number | null };
type HistoryProgress = {
  capture_id: string;
  completed_samples: number;
  completed_chunks: number;
};
const RANGE_LABELS: Record<Exclude<Range, "custom">, string> = {
  all: "All time",
  today: "Today",
  week: "This week",
  days7: "Last 7 days",
  days30: "Last 30 days",
};
const ORIGIN_LABELS: Record<Origin, string> = {
  all: "All sources",
  handy: "Handy Flow",
  wispr: "Wispr Flow",
  manual: "Manually imported",
};

const unwrap = <T,>(
  result: { status: "ok"; data: T } | { status: "error"; error: string },
): T => {
  if (result.status === "ok") return result.data;
  throw new Error(result.error);
};

const localDayStart = (date: Date) =>
  new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime();

const rangeBounds = (range: Range): DateBounds => {
  const now = new Date();
  const tomorrow = localDayStart(
    new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1),
  );
  if (range === "all" || range === "custom")
    return { fromMs: null, toMs: null };
  if (range === "today") return { fromMs: localDayStart(now), toMs: tomorrow };
  if (range === "week") {
    const offset = (now.getDay() + 6) % 7;
    return {
      fromMs: localDayStart(
        new Date(now.getFullYear(), now.getMonth(), now.getDate() - offset),
      ),
      toMs: tomorrow,
    };
  }
  const days = range === "days7" ? 6 : 29;
  return {
    fromMs: localDayStart(
      new Date(now.getFullYear(), now.getMonth(), now.getDate() - days),
    ),
    toMs: tomorrow,
  };
};

const dateInputValue = (value: number | null) => {
  if (value === null) return "";
  const date = new Date(value);
  const month = `${date.getMonth() + 1}`.padStart(2, "0");
  return `${date.getFullYear()}-${month}-${`${date.getDate()}`.padStart(2, "0")}`;
};

const parseDateStart = (value: string) =>
  value ? new Date(`${value}T00:00:00`).getTime() : null;
const parseDateEnd = (value: string) =>
  value
    ? new Date(
        new Date(`${value}T00:00:00`).getTime() + 24 * 60 * 60 * 1000,
      ).getTime()
    : null;

const IconButton: React.FC<{
  onClick: () => void;
  title: string;
  disabled?: boolean;
  active?: boolean;
  children: React.ReactNode;
}> = ({ onClick, title, disabled, active, children }) => (
  <button
    onClick={onClick}
    disabled={disabled}
    className={`p-1.5 rounded-md flex items-center justify-center transition-colors cursor-pointer disabled:cursor-not-allowed disabled:text-text/20 ${
      active
        ? "text-logo-primary hover:text-logo-primary/80"
        : "text-text/50 hover:text-logo-primary"
    }`}
    title={title}
  >
    {children}
  </button>
);

export const HistorySettings: React.FC = () => {
  const { t, i18n } = useTranslation();
  const [entries, setEntries] = useState<CanonicalHistoryEntry[]>([]);
  const [cursor, setCursor] = useState<string | null>(null);
  const cursorRef = useRef<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [importing, setImporting] = useState(false);
  const [dragging, setDragging] = useState(false);
  const [range, setRange] = useState<Range>("all");
  const [origin, setOrigin] = useState<Origin>("all");
  const [bounds, setBounds] = useState<DateBounds>({
    fromMs: null,
    toMs: null,
  });
  const sentinelRef = useRef<HTMLDivElement>(null);
  const dropZoneRef = useRef<HTMLDivElement>(null);
  const entriesRef = useRef<CanonicalHistoryEntry[]>([]);
  const loadingRef = useRef(false);
  const queuedRefreshRef = useRef<boolean | null>(null);
  const loadPageRef = useRef<(reset: boolean, silent?: boolean) => void>(
    () => {},
  );

  const activeBounds = range === "custom" ? bounds : rangeBounds(range);
  const filterKey = `${activeBounds.fromMs}:${activeBounds.toMs}:${origin}`;
  const filterKeyRef = useRef(filterKey);
  filterKeyRef.current = filterKey;
  const loadPage = useCallback(
    async (reset: boolean, silent = false) => {
      if (loadingRef.current) {
        if (reset)
          queuedRefreshRef.current =
            queuedRefreshRef.current === false ? false : silent;
        return;
      }
      loadingRef.current = true;
      if (reset && !silent) setLoading(true);
      try {
        const wanted =
          reset && silent
            ? Math.max(PAGE_SIZE, entriesRef.current.length)
            : PAGE_SIZE;
        const loaded: CanonicalHistoryEntry[] = [];
        let nextCursor = reset ? null : cursorRef.current;
        do {
          const page = unwrap(
            await commands.canonicalHistoryPage(
              {
                from_ms: activeBounds.fromMs,
                to_ms: activeBounds.toMs,
                origin,
              },
              Math.min(200, wanted - loaded.length),
              nextCursor,
            ),
          );
          loaded.push(...page.entries);
          nextCursor = page.next_cursor;
        } while (reset && silent && nextCursor && loaded.length < wanted);
        if (filterKeyRef.current !== filterKey) return;
        const nextEntries = reset ? loaded : [...entriesRef.current, ...loaded];
        entriesRef.current = nextEntries;
        setEntries(nextEntries);
        cursorRef.current = nextCursor;
        setCursor(nextCursor);
      } catch (error) {
        console.error("Failed to load canonical history:", error);
      } finally {
        setLoading(false);
        loadingRef.current = false;
        const queuedRefresh = queuedRefreshRef.current;
        queuedRefreshRef.current = null;
        if (queuedRefresh !== null) loadPageRef.current(true, queuedRefresh);
      }
    },
    [activeBounds.fromMs, activeBounds.toMs, filterKey, origin],
  );
  loadPageRef.current = loadPage;

  useEffect(() => {
    loadPage(true);
  }, [loadPage]);

  useEffect(() => {
    if (loading || !cursor) return;
    const sentinel = sentinelRef.current;
    if (!sentinel) return;
    const observer = new IntersectionObserver(([entry]) => {
      if (entry.isIntersecting && entriesRef.current.length > 0)
        loadPage(false);
    });
    observer.observe(sentinel);
    return () => observer.disconnect();
  }, [cursor, loadPage, loading]);

  useEffect(() => {
    const unlisten = listen("canonical-history-changed", () =>
      loadPage(true, true),
    );
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [loadPage]);

  useEffect(() => {
    const unlisten = listen<HistoryProgress>(
      "canonical-history-progress",
      ({ payload }) => {
        const index = entriesRef.current.findIndex(
          (entry) => entry.capture_id === payload.capture_id,
        );
        if (index < 0) return;
        const entry = entriesRef.current[index];
        if (!["pending", "running"].includes(entry.attempt_status)) return;
        const next = [...entriesRef.current];
        next[index] = {
          ...entry,
          attempt_status: "running",
          completed_samples: payload.completed_samples,
          completed_chunks: payload.completed_chunks,
        };
        entriesRef.current = next;
        setEntries(next);
      },
    );
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const importPaths = useCallback(
    async (paths: string[]) => {
      setImporting(true);
      try {
        for (const path of paths) {
          try {
            unwrap(await commands.importLocalAudio(path));
            toast.success(t("settings.history.importQueued"));
          } catch (error) {
            toast.error(
              `${t("settings.history.importError")}: ${String(error)}`,
            );
          }
        }
      } finally {
        setImporting(false);
        loadPage(true, true);
      }
    },
    [loadPage, t],
  );

  useEffect(() => {
    const unlisten = getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "leave") setDragging(false);
      if (event.payload.type === "leave") return;
      const rect = dropZoneRef.current?.getBoundingClientRect();
      const x = event.payload.position.x / window.devicePixelRatio;
      const y = event.payload.position.y / window.devicePixelRatio;
      const inside =
        !!rect &&
        x >= rect.left &&
        x <= rect.right &&
        y >= rect.top &&
        y <= rect.bottom;
      if (event.payload.type === "enter" || event.payload.type === "over")
        setDragging(inside);
      if (event.payload.type === "drop") {
        setDragging(false);
        if (inside) void importPaths(event.payload.paths);
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [importPaths]);

  const pickAudio = async () => {
    const selected = await open({
      multiple: true,
      directory: false,
      filters: [{ name: "Audio", extensions: ["wav", "mp3"] }],
    });
    if (selected)
      await importPaths(typeof selected === "string" ? [selected] : selected);
  };

  const chooseRange = (next: Range) => {
    setRange(next);
    if (next !== "custom") setBounds(rangeBounds(next));
  };

  const getAudioUrl = useCallback(async (captureId: string) => {
    try {
      const path = unwrap(await commands.canonicalAudioFilePath(captureId));
      return convertFileSrc(path, "asset");
    } catch (error) {
      console.error("Failed to load canonical audio:", error);
      return null;
    }
  }, []);

  const openRecordingsFolder = async () => {
    try {
      unwrap(await commands.openRecordingsFolder());
    } catch (error) {
      console.error("Failed to open recordings folder:", error);
    }
  };

  const groups = entries.reduce<
    Array<{ day: string; entries: CanonicalHistoryEntry[] }>
  >((all, entry) => {
    const day = formatDateMs(entry.created_at_ms, i18n.language);
    const current = all[all.length - 1];
    if (!current || current.day !== day) all.push({ day, entries: [entry] });
    else current.entries.push(entry);
    return all;
  }, []);

  return (
    <div
      ref={dropZoneRef}
      className={`max-w-3xl w-full mx-auto space-y-6 ${dragging ? "ring-2 ring-logo-primary rounded-lg" : ""}`}
    >
      <div className="px-4 flex items-center justify-between">
        <h2 className="text-xs font-medium text-mid-gray uppercase tracking-wide">
          {t("settings.history.title")}
        </h2>
        <div className="flex gap-2">
          <Button
            onClick={pickAudio}
            disabled={importing}
            variant="secondary"
            size="sm"
            className="flex items-center gap-2"
          >
            <Upload className="w-4 h-4" />
            <span>{t("settings.history.importAudio")}</span>
          </Button>
          <Button
            onClick={openRecordingsFolder}
            variant="secondary"
            size="sm"
            className="flex items-center gap-2"
          >
            <FolderOpen className="w-4 h-4" />
            <span>{t("settings.history.openFolder")}</span>
          </Button>
        </div>
      </div>

      <p className="px-4 text-xs text-text/50" role="status">
        {dragging
          ? t("settings.history.dropAudio")
          : t("settings.history.dragAudio")}
      </p>

      <div className="px-4 flex flex-wrap gap-2 items-center">
        {(
          ["all", "today", "week", "days7", "days30"] as Array<
            Exclude<Range, "custom">
          >
        ).map((value) => (
          <Button
            key={value}
            size="sm"
            variant={range === value ? "primary" : "secondary"}
            onClick={() => chooseRange(value)}
          >
            {t(`settings.history.filters.${value}`, {
              defaultValue: RANGE_LABELS[value],
            })}
          </Button>
        ))}
        <select
          aria-label={t("settings.history.filters.origin", {
            defaultValue: "Origin",
          })}
          value={origin}
          onChange={(event) => setOrigin(event.target.value as Origin)}
          className="text-sm bg-background border border-mid-gray/30 rounded px-2 py-1"
        >
          <option value="all">
            {t("settings.history.filters.origins.all", {
              defaultValue: ORIGIN_LABELS.all,
            })}
          </option>
          <option value="handy">
            {t("settings.history.filters.origins.handy", {
              defaultValue: ORIGIN_LABELS.handy,
            })}
          </option>
          <option value="wispr">
            {t("settings.history.filters.origins.wispr", {
              defaultValue: ORIGIN_LABELS.wispr,
            })}
          </option>
          <option value="manual">
            {t("settings.history.filters.origins.manual", {
              defaultValue: ORIGIN_LABELS.manual,
            })}
          </option>
        </select>
        <input
          aria-label={t("settings.history.filters.from", {
            defaultValue: "From date",
          })}
          type="date"
          value={dateInputValue(bounds.fromMs)}
          onChange={(event) => {
            setRange("custom");
            setBounds((previous) => ({
              ...previous,
              fromMs: parseDateStart(event.target.value),
            }));
          }}
          className="text-sm bg-background border border-mid-gray/30 rounded px-2 py-1"
        />
        <input
          aria-label={t("settings.history.filters.to", {
            defaultValue: "To date",
          })}
          type="date"
          value={dateInputValue(
            bounds.toMs === null ? null : bounds.toMs - 24 * 60 * 60 * 1000,
          )}
          onChange={(event) => {
            setRange("custom");
            setBounds((previous) => ({
              ...previous,
              toMs: parseDateEnd(event.target.value),
            }));
          }}
          className="text-sm bg-background border border-mid-gray/30 rounded px-2 py-1"
        />
      </div>

      <div className="bg-background border border-mid-gray/20 rounded-lg overflow-visible">
        {loading ? (
          <div className="px-4 py-3 text-center text-text/60">
            {t("settings.history.loading")}
          </div>
        ) : entries.length === 0 ? (
          <div className="px-4 py-3 text-center text-text/60">
            {t("settings.history.empty")}
          </div>
        ) : (
          <AudioPlayerGroup>
            {groups.map((group) => (
              <section key={group.day}>
                <h3 className="px-4 py-2 text-xs font-medium text-mid-gray uppercase tracking-wide bg-mid-gray/5">
                  {group.day}
                </h3>
                <div className="divide-y divide-mid-gray/20">
                  {group.entries.map((entry) => (
                    <CanonicalHistoryCard
                      key={entry.capture_id}
                      entry={entry}
                      getAudioUrl={getAudioUrl}
                      onChanged={(hard) => loadPage(true, !hard)}
                    />
                  ))}
                </div>
              </section>
            ))}
            <div ref={sentinelRef} className="h-1" />
          </AudioPlayerGroup>
        )}
      </div>
    </div>
  );
};

const CanonicalHistoryCard: React.FC<{
  entry: CanonicalHistoryEntry;
  getAudioUrl: (captureId: string) => Promise<string | null>;
  onChanged: (hard?: boolean) => void;
}> = ({ entry, getAudioUrl, onChanged }) => {
  const { t, i18n } = useTranslation();
  const [copied, setCopied] = useState(false);
  const [retrying, setRetrying] = useState(false);
  const [seams, setSeams] = useState<CanonicalSeamDetail[]>([]);
  const [draft, setDraft] = useState(entry.text);
  const [savingReview, setSavingReview] = useState(false);
  const [editingReview, setEditingReview] = useState(false);
  const reviewLabels = {
    reviewed: t("settings.history.reviewedText"),
    left: t("settings.history.seamBefore"),
    right: t("settings.history.seamAfter"),
    edit: t("settings.history.editReviewedText"),
    save: t("settings.history.confirmText"),
    error: t("settings.history.confirmTextError"),
  };
  const audioCorrupt = entry.integrity_state === "audio_corrupt";
  const pending = entry.integrity_state === "pending_audio";
  const recovered = entry.integrity_state === "recovered_orphan";
  const processing = ["pending", "running"].includes(entry.attempt_status);
  const showChunkProgress = (entry.audio_duration_ms ?? 0) > 6 * 60_000;
  const progress =
    entry.audio_duration_ms && entry.audio_duration_ms > 0
      ? Math.min(
          100,
          Math.round(
            (entry.completed_samples * 100) / (entry.audio_duration_ms * 16),
          ),
        )
      : 0;
  useEffect(() => {
    if (entry.review_seams === 0) {
      setSeams([]);
      return;
    }
    commands
      .canonicalSeamDetails(entry.capture_id)
      .then((result) => setSeams(unwrap(result)))
      .catch((error) =>
        console.error("Could not load transcription seams:", error),
      );
  }, [entry.capture_id, entry.review_seams]);
  useEffect(() => {
    if (!editingReview) setDraft(entry.text);
  }, [entry.text, editingReview]);
  const confirmReview = async () => {
    try {
      setSavingReview(true);
      unwrap(
        await commands.confirmCanonicalHistoryText(entry.capture_id, draft),
      );
      setEditingReview(false);
      onChanged();
    } catch (error) {
      console.error("Could not save reviewed transcript:", error);
      toast.error(reviewLabels.error);
    } finally {
      setSavingReview(false);
    }
  };
  const copy = async () => {
    await navigator.clipboard.writeText(entry.text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };
  const toggleSaved = async () => {
    try {
      unwrap(await commands.toggleCanonicalHistorySaved(entry.capture_id));
      onChanged();
    } catch (error) {
      console.error("Failed to toggle canonical history saved state:", error);
    }
  };
  const trash = async () => {
    try {
      unwrap(await commands.trashCanonicalHistoryEntry(entry.capture_id));
      onChanged(true);
    } catch (error) {
      console.error("Failed to trash canonical history entry:", error);
      toast.error(t("settings.history.deleteError"));
    }
  };
  const retry = async () => {
    try {
      setRetrying(true);
      unwrap(await commands.retryCanonicalHistoryEntry(entry.capture_id));
      onChanged();
    } catch (error) {
      console.error("Failed to retry canonical history entry:", error);
      toast.error(t("settings.history.retranscribeError"));
    } finally {
      setRetrying(false);
    }
  };
  return (
    <div className="px-4 py-2 pb-5 flex flex-col gap-3">
      <div className="flex justify-between items-center gap-2">
        <div className="flex items-center gap-2 min-w-0">
          <p className="text-sm font-medium">
            {formatDateTimeMs(entry.created_at_ms, i18n.language)}
          </p>
          <span
            className={`text-xs px-1.5 py-0.5 rounded ${entry.origin === "wispr" ? "bg-violet-500/15 text-violet-400" : entry.origin === "manual" ? "bg-blue-500/15 text-blue-400" : "bg-mid-gray/15 text-text/60"}`}
          >
            {entry.origin === "wispr"
              ? t("settings.history.filters.origins.wispr", {
                  defaultValue: ORIGIN_LABELS.wispr,
                })
              : entry.origin === "manual"
                ? t("settings.history.filters.origins.manual", {
                    defaultValue: ORIGIN_LABELS.manual,
                  })
                : t("settings.history.filters.origins.handy", {
                    defaultValue: ORIGIN_LABELS.handy,
                  })}
          </span>
        </div>
        <div className="flex items-center">
          <IconButton
            onClick={copy}
            disabled={!entry.text || retrying}
            title={t("settings.history.copyToClipboard")}
          >
            {copied ? <Check width={16} /> : <Copy width={16} />}
          </IconButton>
          <IconButton
            onClick={toggleSaved}
            disabled={retrying}
            active={entry.saved}
            title={
              entry.saved
                ? t("settings.history.unsave")
                : t("settings.history.save")
            }
          >
            <Star width={16} fill={entry.saved ? "currentColor" : "none"} />
          </IconButton>
          <IconButton
            onClick={retry}
            disabled={
              !entry.audio_file_name ||
              audioCorrupt ||
              pending ||
              processing ||
              retrying
            }
            title={t("settings.history.retranscribe")}
          >
            <RotateCcw
              width={16}
              style={
                retrying
                  ? { animation: "spin 1s linear infinite reverse" }
                  : undefined
              }
            />
          </IconButton>
          <IconButton
            onClick={trash}
            disabled={retrying}
            title={t("settings.history.delete")}
          >
            <Trash2 width={16} />
          </IconButton>
        </div>
      </div>
      {entry.source_app && (
        <p className="text-xs text-text/50">{entry.source_app}</p>
      )}
      {entry.origin === "manual" && (
        <p className="text-xs text-text/60 break-all">{entry.title}</p>
      )}
      {entry.attempt_status === "pending" && (
        <p
          className="text-xs text-amber-400 flex items-center gap-1.5"
          role="status"
        >
          <RotateCcw className="w-3.5 h-3.5 animate-spin" aria-hidden="true" />
          {t("settings.history.transcriptionPending")}
        </p>
      )}
      {entry.attempt_status === "running" && (
        <p
          className="text-xs text-amber-400 flex items-center gap-1.5"
          role="status"
        >
          <RotateCcw className="w-3.5 h-3.5 animate-spin" aria-hidden="true" />
          {showChunkProgress && entry.completed_chunks > 0
            ? t("settings.history.progress", {
                progress,
                chunk: entry.completed_chunks + 1,
              })
            : t("settings.history.transcribing")}
        </p>
      )}
      {!entry.text && !processing && (
        <p className="text-xs text-amber-400" role="status">
          {audioCorrupt
            ? t("settings.history.audioCorrupt")
            : pending
              ? t("settings.history.pendingAudio")
              : recovered
                ? t("settings.history.recoveredAudio")
                : t("settings.history.transcriptionFailed")}
        </p>
      )}
      {entry.text && entry.attempt_status === "failed" && (
        <p className="text-xs text-amber-400" role="status">
          {t("settings.history.transcriptionFailed")}
        </p>
      )}
      {entry.attempt_status === "failed" && entry.attempt_error && (
        <p className="text-xs text-text/60 select-text break-words">
          {entry.attempt_error}
        </p>
      )}
      <p className="italic text-sm text-text/90 select-text cursor-text whitespace-pre-wrap break-words">
        {entry.text}
      </p>
      {entry.review_seams > 0 && (
        <details className="text-xs text-amber-400">
          <summary>
            {entry.text_reviewed
              ? reviewLabels.reviewed
              : t("settings.history.reviewSeams", { count: seams.length })}
          </summary>
          {seams.map((seam) => (
            <div key={seam.position_ms} className="mt-2 select-text">
              {Math.floor(seam.position_ms / 60000)}:
              {`${Math.floor((seam.position_ms % 60000) / 1000)}`.padStart(
                2,
                "0",
              )}
              {" · "}
              <p>
                {reviewLabels.left}: <u>{seam.left}</u>
              </p>
              <p>
                {reviewLabels.right}: <u>{seam.right}</u>
              </p>
            </div>
          ))}
          {editingReview ? (
            <div className="mt-3 space-y-2">
              <label htmlFor={`review-${entry.capture_id}`}>
                {reviewLabels.edit}
              </label>
              <textarea
                id={`review-${entry.capture_id}`}
                value={draft}
                onChange={(event) => setDraft(event.target.value)}
                rows={10}
                className="w-full rounded border border-mid-gray/30 bg-background p-2 text-sm text-text"
              />
              <Button
                size="sm"
                onClick={confirmReview}
                disabled={savingReview || !draft.trim()}
              >
                {reviewLabels.save}
              </Button>
            </div>
          ) : (
            <Button
              size="sm"
              variant="secondary"
              className="mt-3"
              onClick={() => setEditingReview(true)}
            >
              {reviewLabels.edit}
            </Button>
          )}
        </details>
      )}
      {entry.audio_file_name && !audioCorrupt && !pending && (
        <AudioPlayer
          onLoadRequest={() => getAudioUrl(entry.capture_id)}
          className="w-full"
        />
      )}
    </div>
  );
};
