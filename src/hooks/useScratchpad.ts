import { useCallback, useEffect, useRef, useState } from "react";
import { commands, type NoteDto, type NoteVersionDto } from "@/bindings";

export type { NoteDto as Note, NoteVersionDto as NoteVersion };

type Res<T> = { status: "ok"; data: T } | { status: "error"; error: string };
function unwrap<T>(r: Res<T>): T {
  if (r.status === "ok") return r.data;
  throw new Error(r.error);
}

const AUTOSAVE_DEBOUNCE_MS = 600;

export interface UseScratchpadReturn {
  /** Active note or null when no note exists yet. */
  note: NoteDto | null;
  /** Latest known content (optimistic; DB state may lag behind). */
  content: string;
  /** dirty = local edits not yet persisted as a version. */
  dirty: boolean;
  saving: boolean;
  lastError: string | null;
  lastSavedVersionNo: number | null;
  setContent: (text: string) => void;
  createNote: () => Promise<NoteDto | null>;
  reload: () => Promise<void>;
  /** Switch the pad to another note (NOTE-304 search results). */
  openNote: (noteId: string) => Promise<void>;
}

/**
 * Scratchpad state with debounced autosave onto the append-only Note
 * domain (PAD-302). Identical content is a backend no-op (hash dedupe),
 * so aggressive debounce flushing can never duplicate versions.
 */
export const useScratchpad = (): UseScratchpadReturn => {
  const [note, setNote] = useState<NoteDto | null>(null);
  const [content, setContentState] = useState("");
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);
  const [lastError, setLastError] = useState<string | null>(null);
  const [lastSavedVersionNo, setLastSavedVersionNo] = useState<number | null>(
    null,
  );

  const noteRef = useRef<NoteDto | null>(null);
  const contentRef = useRef("");
  const savedContentRef = useRef("");
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const inFlightRef = useRef<Promise<void> | null>(null);
  const mountedRef = useRef(true);

  const persist = useCallback(async () => {
    const current = noteRef.current;
    const text = contentRef.current;
    if (!current || text === savedContentRef.current) return;

    const run = async () => {
      setSaving(true);
      try {
        const version = unwrap(
          await commands.notesAppend(current.id, text, "manual_edit"),
        );
        if (!mountedRef.current) return;
        // Only treat as saved if no newer local edit happened meanwhile.
        if (contentRef.current === text) {
          savedContentRef.current = text;
          setDirty(false);
          if (version) setLastSavedVersionNo(version.version_no);
        }
        setLastError(null);
      } catch (e) {
        if (mountedRef.current) {
          setLastError(e instanceof Error ? e.message : String(e));
        }
      } finally {
        if (mountedRef.current) setSaving(false);
      }
    };

    // Chain saves: a pending save completes before the next one starts so
    // versions never race (append-only order = version_no order).
    const prev = inFlightRef.current ?? Promise.resolve();
    const flight = prev.then(run, run);
    inFlightRef.current = flight.finally(() => {
      if (inFlightRef.current === flight) inFlightRef.current = null;
    });
    await flight;
  }, []);

  const scheduleSave = useCallback(() => {
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => {
      timerRef.current = null;
      void persist();
    }, AUTOSAVE_DEBOUNCE_MS);
  }, [persist]);

  /** Flush any pending autosave immediately (used on blur/unmount). */
  const flush = useCallback(async () => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
    await persist();
  }, [persist]);

  const reload = useCallback(async () => {
    try {
      const notes = unwrap(await commands.notesList());
      const active = notes[0] ?? null;
      noteRef.current = active;
      setNote(active);
      if (active) {
        const current = unwrap(await commands.notesCurrent(active.id));
        const text = current?.content ?? "";
        contentRef.current = text;
        savedContentRef.current = text;
        setContentState(text);
        setDirty(false);
        if (current) setLastSavedVersionNo(current.version_no);
      } else {
        contentRef.current = "";
        savedContentRef.current = "";
        setContentState("");
        setDirty(false);
        setLastSavedVersionNo(null);
      }
      setLastError(null);
    } catch (e) {
      setLastError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  useEffect(() => {
    mountedRef.current = true;
    void reload();
    return () => {
      mountedRef.current = false;
    };
  }, [reload]);

  const setContent = useCallback(
    (text: string) => {
      contentRef.current = text;
      setContentState(text);
      setDirty(text !== savedContentRef.current);
      scheduleSave();
    },
    [scheduleSave],
  );

  /**
   * Open another note. Pending edits are flushed FIRST: the debounced save
   * targets `noteRef.current`, so switching with a timer still armed would
   * write the outgoing text onto the incoming note.
   */
  const openNote = useCallback(
    async (noteId: string) => {
      await flush();
      try {
        const notes = unwrap(await commands.notesList());
        const target = notes.find((n) => n.id === noteId) ?? null;
        if (!target) return;
        const current = unwrap(await commands.notesCurrent(target.id));
        const text = current?.content ?? "";
        noteRef.current = target;
        contentRef.current = text;
        savedContentRef.current = text;
        if (!mountedRef.current) return;
        setNote(target);
        setContentState(text);
        setDirty(false);
        setLastSavedVersionNo(current ? current.version_no : null);
        setLastError(null);
      } catch (e) {
        if (mountedRef.current) {
          setLastError(e instanceof Error ? e.message : String(e));
        }
      }
    },
    [flush],
  );

  const createNote = useCallback(async (): Promise<NoteDto | null> => {
    try {
      const created = unwrap(
        await commands.notesCreate("Note", contentRef.current),
      );
      noteRef.current = created;
      setNote(created);
      savedContentRef.current = contentRef.current;
      setDirty(false);
      setLastError(null);
      return created;
    } catch (e) {
      setLastError(e instanceof Error ? e.message : String(e));
      return null;
    }
  }, []);

  return {
    note,
    content,
    dirty,
    saving,
    lastError,
    lastSavedVersionNo,
    setContent,
    createNote,
    reload,
    openNote,
  };
};
