import { useCallback, useEffect, useState } from "react";
import { commands, type NoteDto, type NoteVersionDto } from "@/bindings";
import { useScratchpad } from "@/hooks/useScratchpad";
import { useTranslation } from "react-i18next";
import { hideScratchpad } from "./actions";

/** Bounded page size for note search (backend clamps to 1..=200 anyway). */
const SEARCH_LIMIT = 20;
const SEARCH_DEBOUNCE_MS = 200;

/**
 * Floating Markdown scratchpad (PAD-302) with dictation target and
 * PromptProfile transforms (PAD-303), plus note search and version
 * restore (NOTE-304): plain text/Markdown only, autosaved onto
 * append-only note versions. Transforms append the LLM output as a
 * NEW version (source = `transform`); the source version stays restorable.
 */
export default function ScratchpadWindow() {
  const { t } = useTranslation();
  const {
    note,
    content,
    dirty,
    saving,
    lastError,
    setContent,
    createNote,
    openNote,
  } = useScratchpad();
  const [versionsOpen, setVersionsOpen] = useState(false);
  const [versions, setVersions] = useState<NoteVersionDto[]>([]);
  const [searchOpen, setSearchOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<NoteDto[]>([]);
  const [transforms, setTransforms] = useState<
    Array<{ id: string; name: string }>
  >([]);
  const [transformBusy, setTransformBusy] = useState(false);
  const [transformError, setTransformError] = useState<string | null>(null);

  const loadVersions = useCallback(async () => {
    if (!note) return;
    try {
      const list = await commands.notesVersions(note.id);
      if (list.status === "ok") setVersions(list.data);
    } catch {
      setVersions([]);
    }
  }, [note]);

  useEffect(() => {
    if (versionsOpen) void loadVersions();
  }, [versionsOpen, loadVersions]);

  useEffect(() => {
    let cancelled = false;
    const load = async () => {
      try {
        const profiles = await commands.promptProfilesList("transform");
        if (!cancelled && profiles.status === "ok") {
          setTransforms(profiles.data.map((p) => ({ id: p.id, name: p.name })));
        }
      } catch {
        /* transforms stay empty; feature degrades to plain pad */
      }
    };
    void load();
    return () => {
      cancelled = true;
    };
  }, []);

  const applyTransform = useCallback(
    async (profileId: string) => {
      if (!note || transformBusy) return;
      setTransformBusy(true);
      setTransformError(null);
      try {
        const result = await commands.notesTransform(note.id, profileId);
        if (result.status === "ok") {
          setContent(result.data.content);
          void loadVersions();
        } else {
          setTransformError(result.error);
        }
      } catch (e) {
        setTransformError(e instanceof Error ? e.message : String(e));
      } finally {
        setTransformBusy(false);
      }
    },
    [note, transformBusy, setContent, loadVersions],
  );

  // Debounced search; an empty query clears results instead of listing
  // everything, so the panel never becomes a full note dump.
  useEffect(() => {
    if (!searchOpen) return;
    const term = query.trim();
    if (!term) {
      setResults([]);
      return;
    }
    let cancelled = false;
    const timer = setTimeout(() => {
      void (async () => {
        try {
          const r = await commands.notesSearch(term, SEARCH_LIMIT, 0);
          if (!cancelled && r.status === "ok") setResults(r.data);
        } catch {
          if (!cancelled) setResults([]);
        }
      })();
    }, SEARCH_DEBOUNCE_MS);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [query, searchOpen]);

  const openResult = useCallback(
    async (noteId: string) => {
      await openNote(noteId);
      setSearchOpen(false);
      setQuery("");
      setResults([]);
      if (versionsOpen) void loadVersions();
    },
    [openNote, versionsOpen, loadVersions],
  );

  const restoreVersion = useCallback(
    async (versionNo: number) => {
      if (!note) return;
      try {
        await commands.notesRestoreVersion(note.id, versionNo);
        await loadVersions();
        const current = await commands.notesCurrent(note.id);
        if (current.status === "ok" && current.data) {
          setContent(current.data.content);
        }
      } catch {
        /* error surfaces via lastError on next save */
      }
    },
    [note, loadVersions, setContent],
  );

  return (
    <div
      className="flex h-full flex-col bg-hf-dark-600 text-hf-ghost-200"
      data-testid="scratchpad-root"
    >
      <header
        className="flex items-center justify-between gap-2 px-3 py-2 bg-hf-dark-500 select-none"
        data-tauri-drag-region
      >
        <span className="text-sm font-medium truncate">
          {note?.title ?? t("scratchpad.title")}
        </span>
        <span className="text-xs opacity-60" data-testid="scratchpad-status">
          {saving
            ? t("scratchpad.saving")
            : dirty
              ? t("scratchpad.pending")
              : t("scratchpad.saved")}
        </span>
      </header>
      {searchOpen && (
        <div
          className="border-b border-hf-dark-400/60"
          data-testid="scratchpad-search"
        >
          <input
            className="w-full bg-transparent px-3 py-1.5 text-sm outline-none"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder={t("scratchpad.searchPlaceholder")}
            spellCheck={false}
            autoFocus
            data-testid="scratchpad-search-input"
          />
          {query.trim() !== "" && (
            <ul
              className="max-h-40 overflow-auto"
              data-testid="scratchpad-search-results"
            >
              {results.length === 0 ? (
                <li className="px-3 py-1 text-xs opacity-60">
                  {t("scratchpad.searchEmpty")}
                </li>
              ) : (
                results.map((r) => (
                  <li key={r.id}>
                    <button
                      className="flex w-full items-center gap-2 px-3 py-1 text-left text-xs hover:bg-hf-dark-500"
                      onClick={() => void openResult(r.id)}
                    >
                      {r.pinned && <span aria-hidden="true">📌</span>}
                      <span className="truncate">{r.title}</span>
                    </button>
                  </li>
                ))
              )}
            </ul>
          )}
        </div>
      )}
      {note ? (
        <textarea
          className="flex-1 resize-none bg-transparent px-3 py-2 text-sm font-mono outline-none resize-none"
          value={content}
          onChange={(e) => setContent(e.target.value)}
          placeholder={t("scratchpad.placeholder")}
          spellCheck={false}
          data-testid="scratchpad-editor"
        />
      ) : (
        <div className="flex flex-1 flex-col items-center justify-center gap-3 px-6">
          <p className="text-sm opacity-70 text-center">
            {t("scratchpad.empty")}
          </p>
          <button
            className="px-3 py-1.5 rounded bg-hf-primary-500 text-hf-ghost-100 text-sm"
            onClick={() => void createNote()}
            data-testid="scratchpad-create"
          >
            {t("scratchpad.create")}
          </button>
        </div>
      )}
      {(lastError || transformError) && (
        <p
          className="px-3 pb-1 text-xs text-red-400"
          data-testid="scratchpad-error"
        >
          {transformError ?? lastError}
        </p>
      )}
      {transforms.length > 0 && note && (
        <div
          className="flex flex-wrap gap-1 px-3 py-1.5 border-t border-hf-dark-400/60"
          data-testid="scratchpad-transforms"
        >
          {transforms.map((p) => (
            <button
              key={p.id}
              className="px-2 py-0.5 text-xs rounded bg-hf-dark-400 hover:bg-hf-dark-300 disabled:opacity-50"
              disabled={transformBusy}
              onClick={() => void applyTransform(p.id)}
            >
              {p.name}
            </button>
          ))}
          {transformBusy && (
            <span className="text-xs opacity-60 self-center">
              {t("scratchpad.transformRunning")}
            </span>
          )}
        </div>
      )}
      <footer className="flex items-center justify-between gap-2 px-3 py-1.5 border-t border-hf-dark-400/60">
        <div className="flex items-center gap-3">
          <button
            className="text-xs opacity-80 hover:opacity-100"
            onClick={() => setVersionsOpen((v) => !v)}
            data-testid="scratchpad-versions-toggle"
          >
            {t("scratchpad.versions")}
          </button>
          <button
            className="text-xs opacity-80 hover:opacity-100"
            onClick={() => setSearchOpen((v) => !v)}
            data-testid="scratchpad-search-toggle"
          >
            {t("scratchpad.search")}
          </button>
        </div>
        <button
          className="text-xs opacity-80 hover:opacity-100"
          onClick={() => void hideScratchpad()}
          data-testid="scratchpad-hide"
        >
          {t("scratchpad.hide")}
        </button>
      </footer>
      {versionsOpen && (
        <ul
          className="max-h-40 overflow-auto border-t border-hf-dark-400/60"
          data-testid="scratchpad-versions"
        >
          {versions.map((v) => (
            <li
              key={v.id}
              className="flex items-center justify-between px-3 py-1 text-xs"
            >
              <span className="opacity-70">
                #{v.version_no} · {v.source}
              </span>
              <button
                className="text-hf-primary-400"
                onClick={() => void restoreVersion(v.version_no)}
              >
                {t("scratchpad.restore")}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
