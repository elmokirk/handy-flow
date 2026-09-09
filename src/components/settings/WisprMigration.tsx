import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-dialog";
import { Database, FileInput } from "lucide-react";
import { commands, type ImportReport } from "../../bindings";
import { SettingContainer } from "../ui/SettingContainer";
import { Button } from "../ui/Button";

type Res<T> = { status: "ok"; data: T } | { status: "error"; error: string };
function unwrap<T>(r: Res<T>): T {
  if (r.status === "ok") return r.data;
  throw new Error(r.error);
}

async function pickWisprDb(): Promise<string | null> {
  // Native picker via the dialog plugin (capability dialog:default is set):
  // returns a real filesystem path on every platform, unlike a webview
  // <input type="file"> which never exposes one.
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [
      { name: "Wispr Flow database", extensions: ["sqlite", "db", "sqlite3"] },
      { name: "All files", extensions: ["*"] },
    ],
  });
  return typeof selected === "string" ? selected : null;
}

export const WisprMigration: React.FC<{ grouped?: boolean }> = ({
  grouped = false,
}) => {
  const { t } = useTranslation();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [dry, setDry] = useState<ImportReport | null>(null);
  const [done, setDone] = useState<ImportReport | null>(null);
  const [path, setPath] = useState("");

  const pick = async () => {
    setError("");
    setDry(null);
    setDone(null);
    const p = await pickWisprDb();
    if (!p) return;
    setPath(p);
    setBusy(true);
    try {
      setDry(unwrap(await commands.wisprDryRun(p)));
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const runImport = async () => {
    if (!path) return;
    setBusy(true);
    setError("");
    try {
      setDone(unwrap(await commands.wisprRunImport(path)));
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const counts = (r: ImportReport) =>
    [
      ["history", r.history_imported],
      ["audio", r.audio_extracted],
      ["dictionary", r.dictionary_imported],
      ["snippets", r.snippet_triggers_imported],
      ["polish", r.polish_imported],
    ] as const;

  return (
    <SettingContainer
      title={t("settings.wisprMigration.title")}
      description={t("settings.wisprMigration.description")}
      descriptionMode="inline"
      grouped={grouped}
      layout="stacked"
    >
      <div className="flex flex-col gap-3 w-full">
        <div className="flex items-center gap-2">
          <Button variant="secondary" size="sm" onClick={pick}>
            <span className="flex items-center gap-2">
              <FileInput size={14} />
              {t("settings.wisprMigration.pickButton")}
            </span>
          </Button>
          {path && (
            <span className="text-xs text-text/60 truncate max-w-[240px] font-mono">
              {path}
            </span>
          )}
        </div>

        {busy && (
          <div className="text-sm text-text/70">
            {t("settings.wisprMigration.working")}
          </div>
        )}

        {error && (
          <div className="text-sm text-red-400 border border-red-500/40 bg-red-500/10 rounded px-3 py-2">
            {error}
          </div>
        )}

        {dry && !done && (
          <div className="flex flex-col gap-2 w-full">
            <div className="text-sm text-text/80 flex items-center gap-2">
              <Database size={14} />
              {t("settings.wisprMigration.dryRunTitle")}
            </div>
            <ul className="text-sm text-text/70 grid grid-cols-2 gap-x-6 gap-y-1">
              {counts(dry).map(([key, n]) => (
                <li key={key}>
                  {t(`settings.wisprMigration.counts.${key}`, {
                    defaultValue: key,
                  })}
                  : <b>{n}</b>
                </li>
              ))}
            </ul>
            <div>
              <Button variant="primary" size="sm" onClick={runImport}>
                {t("settings.wisprMigration.importButton")}
              </Button>
            </div>
          </div>
        )}

        {done && (
          <div className="text-sm text-hf-primary-500">
            {t("settings.wisprMigration.done")}
          </div>
        )}
      </div>
    </SettingContainer>
  );
};
