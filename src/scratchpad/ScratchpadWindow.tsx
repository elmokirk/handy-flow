import { useCallback, useEffect, useState } from "react";
import { commands, type NoteVersionDto } from "@/bindings";
import { useScratchpad } from "@/hooks/useScratchpad";
import { useTranslation } from "react-i18next";
import { hideScratchpad } from "./actions";

/**
 * Floating Markdown scratchpad (PAD-302): one autosaved plain-text/
 * Markdown note, floating always-on-top window. Plain text only — no
 * Rich Text by contract.
 */
export default function ScratchpadWindow() {
  const { t } = useTranslation();
  const { note, content, dirty, saving, lastError, setContent, createNote } =
    useScratchpad();
  const [versionsOpen, setVersionsOpen] = useState(false);
  const [versions, setVersions] = useState<NoteVersionDto[]>([]);

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
      {lastError && (
        <p
          className="px-3 pb-1 text-xs text-red-400"
          data-testid="scratchpad-error"
        >
          {lastError}
        </p>
      )}
      <footer className="flex items-center justify-between px-3 py-1.5 border-t border-hf-dark-400/60">
        <button
          className="text-xs opacity-80 hover:opacity-100"
          onClick={() => setVersionsOpen((v) => !v)}
          data-testid="scratchpad-versions-toggle"
        >
          {t("scratchpad.versions")}
        </button>
        <button
          className="text-xs opacity-80 hover:opacity-100"
          onClick={() => void commands.hideScratchpad()}
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
