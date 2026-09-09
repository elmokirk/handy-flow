import { useTranslation } from "react-i18next";
import { useCallback, useEffect, useState } from "react";
import { Trash2 } from "lucide-react";
import {
  commands,
  type DictionaryEntry,
  type SnippetRecord,
  type PromptProfileRecord,
} from "../../../bindings";

type Tab = "dictionary" | "snippets" | "profiles";

type Res<T> = { status: "ok"; data: T } | { status: "error"; error: string };
function unwrap<T>(r: Res<T>): T {
  if (r.status === "ok") return r.data;
  throw new Error(r.error);
}

const inputClass =
  "border border-hf-dark-400 bg-hf-dark-500 rounded px-2 py-1 text-sm text-hf-ghost-200 w-full";
const btn =
  "px-3 py-1 text-sm rounded bg-hf-dark-400 hover:bg-hf-dark-300 text-hf-ghost-200 transition-colors";

/* Icon-only delete affordance: red hover target, white trash glyph, no text. */
const deleteBtn =
  "shrink-0 inline-flex items-center justify-center w-7 h-7 rounded text-red-400/70 hover:text-white hover:bg-red-500/80 transition-colors";

const TAB_LABEL_KEYS: Record<Tab, string> = {
  dictionary: "knowledge.tabs.dictionary",
  snippets: "knowledge.tabs.snippets",
  profiles: "knowledge.tabs.profiles",
};

export function KnowledgeSettings() {
  const { t } = useTranslation();
  const [tab, setTab] = useState<Tab>("dictionary");
  const [entries, setEntries] = useState<DictionaryEntry[]>([]);
  const [snippets, setSnippets] = useState<SnippetRecord[]>([]);
  const [profiles, setProfiles] = useState<PromptProfileRecord[]>([]);
  const [term, setTerm] = useState("");
  const [aliases, setAliases] = useState("");
  const [trigger, setTrigger] = useState("");
  const [replacement, setReplacement] = useState("");
  const [importMsg, setImportMsg] = useState("");
  const [errorMsg, setErrorMsg] = useState("");

  const load = useCallback(async () => {
    setEntries(unwrap(await commands.dictionaryList()));
    setSnippets(unwrap(await commands.snippetsList()));
    setProfiles(unwrap(await commands.promptProfilesList("style")));
  }, []);

  // One-time legacy import on mount only. Running it inside load() resurrected
  // deleted terms: the upsert re-imported settings custom_words on every
  // reload, so the delete + immediate refresh appeared as a no-op.
  useEffect(() => {
    void (async () => {
      try {
        const n = unwrap(await commands.dictionaryImportLegacy());
        if (n > 0) setEntries(unwrap(await commands.dictionaryList()));
      } catch {
        /* settings without legacy words */
      }
      void load();
    })();
  }, [load]);

  // Surface command failures instead of silently swallowing them: every
  // mutation routes through run() so a failed delete shows why.
  const run = async (action: () => Promise<void>) => {
    setErrorMsg("");
    try {
      await action();
    } catch (e) {
      setErrorMsg(e instanceof Error ? e.message : String(e));
    }
  };

  const saveDict = () =>
    run(async () => {
      if (!term.trim()) return;
      unwrap(
        await commands.dictionaryUpsert(
          term.trim(),
          aliases
            .split(",")
            .map((a) => a.trim())
            .filter(Boolean),
          true,
        ),
      );
      setTerm("");
      setAliases("");
      setEntries(unwrap(await commands.dictionaryList()));
    });

  const saveSnippet = () =>
    run(async () => {
      if (!trigger.trim() || !replacement.trim()) return;
      unwrap(
        await commands.snippetsUpsert(trigger.trim(), replacement, 0, true),
      );
      setTrigger("");
      setReplacement("");
      setSnippets(unwrap(await commands.snippetsList()));
    });

  const importLegacy = () =>
    run(async () => {
      const n = unwrap(await commands.dictionaryImportLegacy());
      setImportMsg(`${n} imported`);
      setEntries(unwrap(await commands.dictionaryList()));
    });

  const exportCopy = () =>
    run(async () => {
      await navigator.clipboard.writeText(
        unwrap(await commands.dictionaryExport()),
      );
      setImportMsg("copied");
    });

  return (
    <div className="flex flex-col gap-4 w-full">
      {/* Tab bar: flush-left, full width, animated active background so the
          active tab is readable at a glance and content below never shifts
          horizontally when tabs change. */}
      <div
        role="tablist"
        className="flex w-full gap-1 p-1 rounded-lg bg-surface1 border border-mid-gray/20"
      >
        {(["dictionary", "snippets", "profiles"] as Tab[]).map((tb) => (
          <button
            key={tb}
            role="tab"
            aria-selected={tab === tb}
            onClick={() => setTab(tb)}
            className={`flex-1 px-3 py-1.5 rounded-md text-sm transition-colors ${
              tab === tb
                ? "bg-hf-primary-500 text-white font-medium shadow-sm"
                : "text-text/70 hover:bg-mid-gray/10 hover:text-text"
            }`}
          >
            {t(TAB_LABEL_KEYS[tb])}
          </button>
        ))}
      </div>

      {errorMsg && (
        <div className="w-full text-sm text-red-400 border border-red-500/40 bg-red-500/10 rounded px-3 py-2">
          {errorMsg}
        </div>
      )}

      <div className="w-full">
        {tab === "dictionary" && (
          <section className="flex flex-col gap-2 w-full">
            <div className="flex gap-2">
              <input
                className={inputClass}
                placeholder={t("knowledge.term")}
                value={term}
                onChange={(e) => setTerm(e.target.value)}
              />
              <input
                className={inputClass}
                placeholder={t("knowledge.aliases")}
                value={aliases}
                onChange={(e) => setAliases(e.target.value)}
              />
              <button className={btn} onClick={saveDict}>
                {t("knowledge.add")}
              </button>
            </div>
            <div className="flex gap-2">
              <button className={btn} onClick={importLegacy}>
                {t("knowledge.importCustomWords")}
              </button>
              <button className={btn} onClick={exportCopy}>
                {t("knowledge.exportCopy")}
              </button>
              <span className="text-xs opacity-70 self-center">
                {importMsg}
              </span>
            </div>
            <ul className="flex flex-col gap-1 max-h-80 overflow-auto w-full">
              {entries.map((e) => (
                <li
                  key={e.id}
                  className="flex items-center justify-between bg-hf-dark-600/60 rounded px-3 py-1.5 w-full"
                >
                  <span className="min-w-0 truncate">
                    <b>{e.term}</b>
                    {e.aliases.length > 0 && (
                      <i className="opacity-60"> ({e.aliases.join(", ")})</i>
                    )}
                  </span>
                  <button
                    className={deleteBtn}
                    aria-label={t("knowledge.delete")}
                    title={t("knowledge.delete")}
                    onClick={() =>
                      run(async () => {
                        unwrap(await commands.dictionaryDelete(e.id));
                        setEntries(unwrap(await commands.dictionaryList()));
                      })
                    }
                  >
                    <Trash2 size={14} />
                  </button>
                </li>
              ))}
            </ul>
          </section>
        )}

        {tab === "snippets" && (
          <section className="flex flex-col gap-2 w-full">
            <div className="flex gap-2">
              <input
                className={inputClass}
                placeholder={t("knowledge.triggerPhrase")}
                value={trigger}
                onChange={(e) => setTrigger(e.target.value)}
              />
              <input
                className={inputClass}
                placeholder={t("knowledge.replacement")}
                value={replacement}
                onChange={(e) => setReplacement(e.target.value)}
              />
              <button className={btn} onClick={saveSnippet}>
                {t("knowledge.add")}
              </button>
            </div>
            <ul className="flex flex-col gap-1 max-h-80 overflow-auto w-full">
              {snippets.map((s) => (
                <li
                  key={s.id}
                  className="flex items-center justify-between bg-hf-dark-600/60 rounded px-3 py-1.5 w-full"
                >
                  <span className="min-w-0 truncate">
                    <code>{s.trigger}</code> → <b>{s.replacement}</b>
                  </span>
                  <button
                    className={deleteBtn}
                    aria-label={t("knowledge.delete")}
                    title={t("knowledge.delete")}
                    onClick={() =>
                      run(async () => {
                        unwrap(await commands.snippetsDelete(s.id));
                        setSnippets(unwrap(await commands.snippetsList()));
                      })
                    }
                  >
                    <Trash2 size={14} />
                  </button>
                </li>
              ))}
            </ul>
          </section>
        )}

        {tab === "profiles" && (
          <ul className="flex flex-col gap-1 max-h-80 overflow-auto w-full">
            {profiles.map((p) => (
              <li
                key={p.id}
                className="flex items-center justify-between bg-hf-dark-600/60 rounded px-3 py-1.5 w-full"
              >
                <span className="min-w-0 truncate">
                  {p.name} · {p.model} {p.enabled ? "" : "(off)"}
                </span>
                <button
                  className={deleteBtn}
                  aria-label={t("knowledge.delete")}
                  title={t("knowledge.delete")}
                  onClick={() =>
                    run(async () => {
                      unwrap(await commands.promptProfilesDelete(p.id));
                      setProfiles(
                        unwrap(await commands.promptProfilesList("style")),
                      );
                    })
                  }
                >
                  <Trash2 size={14} />
                </button>
              </li>
            ))}
            {profiles.length === 0 && (
              <li className="text-xs opacity-60">
                {t("knowledge.profilesEmpty")}
              </li>
            )}
          </ul>
        )}
      </div>
    </div>
  );
}