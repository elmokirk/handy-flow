import { useTranslation } from "react-i18next";
import { useCallback, useEffect, useState } from "react";
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
  "border border-border bg-crust rounded px-2 py-1 text-sm text-text w-full";
const btn =
  "px-3 py-1 text-sm rounded bg-surface1 hover:bg-surface2 text-text transition-colors";

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

  const load = useCallback(async () => {
    setEntries(unwrap(await commands.dictionaryList()));
    setSnippets(unwrap(await commands.snippetsList()));
    setProfiles(unwrap(await commands.promptProfilesList("style")));
    try {
      (unwrap(await commands.dictionaryImportLegacy())) > 0 &&
        setEntries(unwrap(await commands.dictionaryList()));
    } catch {
      /* settings without legacy words */
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const saveDict = async () => {
    if (!term.trim()) return;
    unwrap(
      await commands.dictionaryUpsert(
        term.trim(),
        aliases.split(",").map((a) => a.trim()).filter(Boolean),
        true,
      ),
    );
    setTerm("");
    setAliases("");
    void load();
  };

  const saveSnippet = async () => {
    if (!trigger.trim() || !replacement.trim()) return;
    unwrap(await commands.snippetsUpsert(trigger.trim(), replacement, 0, true));
    setTrigger("");
    setReplacement("");
    setSnippets(unwrap(await commands.snippetsList()));
  };

  const importLegacy = async () => {
    const n = unwrap(await commands.dictionaryImportLegacy());
    setImportMsg(`${n} imported`);
    void load();
  };

  return (
    <div className="flex flex-col gap-4">
      <div className="flex gap-2">
        {(["dictionary", "snippets", "profiles"] as Tab[]).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`px-3 py-1 rounded text-sm capitalize ${
              tab === t ? "bg-accent text-text" : "bg-surface1 hover:bg-surface2"
            }`}
          >
            {t}
          </button>
        ))}
      </div>

      {tab === "dictionary" && (
        <section className="flex flex-col gap-2">
          <div className="flex gap-2">
            <input className={inputClass} placeholder={t("knowledge.term")} value={term} onChange={(e) => setTerm(e.target.value)} />
            <input className={inputClass} placeholder={t("knowledge.aliases")} value={aliases} onChange={(e) => setAliases(e.target.value)} />
            <button className="px-3 py-1 rounded bg-accent text-text" onClick={saveDict}>{t('knowledge.add')}</button>
          </div>
          <div className="flex gap-2">
            <button className="px-3 py-1 rounded bg-surface1" onClick={importLegacy}>{t('knowledge.importCustomWords')}</button>
            <button
              className="px-3 py-1 rounded bg-surface1"
              onClick={async () => {
                await navigator.clipboard.writeText(unwrap(await commands.dictionaryExport()));
                setImportMsg("copied");
              }}
            >{t('knowledge.exportCopy')}</button>
            <span className="text-xs opacity-70 self-center">{importMsg}</span>
          </div>
          <ul className="flex flex-col gap-1 max-h-80 overflow-auto">
            {entries.map((e) => (
              <li key={e.id} className="flex items-center justify-between bg-surface1/50 rounded px-2 py-1">
                <span>
                  <b>{e.term}</b>
                  {e.aliases.length > 0 && <i className="opacity-60"> ({e.aliases.join(", ")})</i>}
                </span>
                <button
                  className="text-red-400 text-xs"
                  onClick={async () => {
                    unwrap(await commands.dictionaryDelete(e.id));
                    void load();
                  }}
                >{t('knowledge.delete')}</button>
              </li>
            ))}
          </ul>
        </section>
      )}

      {tab === "snippets" && (
        <section className="flex flex-col gap-2">
          <div className="flex gap-2">
            <input className={inputClass} placeholder={t("knowledge.triggerPhrase")} value={trigger} onChange={(e) => setTrigger(e.target.value)} />
            <input className={inputClass} placeholder={t("knowledge.replacement")} value={replacement} onChange={(e) => setReplacement(e.target.value)} />
            <button className="px-3 py-1 rounded bg-accent" onClick={saveSnippet}>{t('knowledge.add')}</button>
          </div>
          <ul className="flex flex-col gap-1 max-h-80 overflow-auto">
            {snippets.map((s) => (
              <li key={s.id} className="flex items-center justify-between bg-surface1/50 rounded px-2 py-1">
                <span>
                  <code>{s.trigger}</code> → <b>{s.replacement}</b>
                </span>
                <button className="text-red-400 text-xs" onClick={async () => { unwrap(await commands.snippetsDelete(s.id)); void load(); }}>{t('knowledge.delete')}</button>
              </li>
            ))}
          </ul>
        </section>
      )}

      {tab === "profiles" && (
        <ul className="flex flex-col gap-1 max-h-80 overflow-auto">
          {profiles.map((p) => (
            <li key={p.id} className="flex items-center justify-between bg-surface1/50 rounded px-2 py-1">
              <span>
                {p.name} · {p.model} {p.enabled ? "" : "(off)"}
              </span>
              <button className="text-red-400 text-xs" onClick={async () => { unwrap(await commands.promptProfilesDelete(p.id)); void load(); }}>
                {t('knowledge.delete')}
              </button>
            </li>
          ))}
          {profiles.length === 0 && (
            <li className="text-xs opacity-60">{t('knowledge.profilesEmpty')}</li>
          )}
        </ul>
      )}
    </div>
  );
}