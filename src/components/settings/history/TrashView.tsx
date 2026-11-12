import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "@/hooks/useSettings";
import { useTrash } from "@/hooks/useTrash";
import { formatDateTimeMs } from "@/utils/dateFormat";
import { Button } from "../../ui/Button";

type Action = { type: "restore" | "purge"; ids: string[] };
export const TrashView: React.FC<{ onClose: () => void }> = ({ onClose }) => {
  const { t, i18n } = useTranslation();
  const { settings } = useSettings();
  const trash = useTrash();
  const [confirmation, setConfirmation] = useState<Action | null>(null);
  const ask = (type: Action["type"], ids: string[]) =>
    setConfirmation({ type, ids });
  return (
    <div className="max-w-3xl w-full mx-auto space-y-4 px-4">
      <div className="flex justify-between items-center">
        <h2>{t("settings.history.trash.title")}</h2>
        <Button variant="secondary" size="sm" onClick={onClose}>
          {t("settings.history.trash.back")}
        </Button>
      </div>
      <p>{t("settings.history.trash.total", { count: trash.data.total })}</p>
      {trash.restored &&
        settings?.recording_retention_period === "preserve_limit" && (
          <p role="status">{t("settings.history.trash.retentionHint")}</p>
        )}
      {trash.error && (
        <div role="alert" className="text-red-500 whitespace-pre-wrap">
          {t("settings.history.trash.error")}: {trash.error}{" "}
          <Button
            size="sm"
            variant="secondary"
            onClick={() => void trash.load()}
          >
            {t("settings.history.trash.retry")}
          </Button>
        </div>
      )}
      <div className="flex gap-2">
        <Button
          size="sm"
          disabled={!trash.selected.length || trash.busy}
          onClick={() => ask("restore", trash.selected)}
        >
          {t("settings.history.trash.restoreSelected")}
        </Button>
        <Button
          size="sm"
          variant="danger"
          disabled={!trash.selected.length || trash.busy}
          onClick={() => ask("purge", trash.selected)}
        >
          {t("settings.history.trash.purgeSelected")}
        </Button>
      </div>
      {trash.loading ? (
        <p>{t("settings.history.loading")}</p>
      ) : trash.data.entries.length === 0 && !trash.error ? (
        <p>{t("settings.history.trash.empty")}</p>
      ) : (
        <ul className="space-y-2">
          {trash.data.entries.map((entry) => (
            <li
              key={entry.capture_id}
              className="border border-mid-gray/20 rounded p-3 flex flex-wrap items-center gap-3"
            >
              <input
                type="checkbox"
                aria-label={t("settings.history.trash.select", {
                  title: entry.title,
                })}
                checked={trash.selected.includes(entry.capture_id)}
                onChange={() => trash.toggle(entry.capture_id)}
              />
              <div className="flex-1 min-w-0">
                <p className="truncate">{entry.title}</p>
                <p className="text-xs">
                  {formatDateTimeMs(entry.created_at_ms, i18n.language)} ·{" "}
                  {t("settings.history.trash.badge")}
                </p>
              </div>
              <Button
                size="sm"
                variant="secondary"
                disabled={trash.busy}
                onClick={() => void trash.act("restore", [entry.capture_id])}
              >
                {t("settings.history.trash.restore")}
              </Button>
              <Button
                size="sm"
                variant="danger"
                disabled={trash.busy}
                onClick={() => ask("purge", [entry.capture_id])}
              >
                {t("settings.history.trash.purge")}
              </Button>
            </li>
          ))}
        </ul>
      )}
      <div className="flex items-center gap-2">
        <Button
          size="sm"
          variant="secondary"
          disabled={trash.page === 0 || trash.busy}
          onClick={() => trash.changePage(trash.page - 1)}
        >
          {t("settings.history.trash.previous")}
        </Button>
        <span>
          {t("settings.history.trash.page", { page: trash.page + 1 })}
        </span>
        <Button
          size="sm"
          variant="secondary"
          disabled={
            (trash.page + 1) * trash.pageSize >= trash.data.total || trash.busy
          }
          onClick={() => trash.changePage(trash.page + 1)}
        >
          {t("settings.history.trash.next")}
        </Button>
      </div>
      {confirmation && (
        <div
          role="dialog"
          aria-modal="true"
          aria-labelledby="trash-confirm-title"
          className="fixed inset-0 z-50 bg-black/60 flex items-center justify-center p-4"
        >
          <div className="bg-background border border-mid-gray/30 rounded-lg p-5 space-y-4">
            <h3 id="trash-confirm-title">
              {t(
                confirmation.type === "purge"
                  ? "settings.history.trash.confirmPurge"
                  : "settings.history.trash.confirmRestore",
                { count: confirmation.ids.length },
              )}
            </h3>
            <div className="flex gap-2">
              <Button variant="secondary" onClick={() => setConfirmation(null)}>
                {t("settings.history.trash.cancel")}
              </Button>
              <Button
                variant={confirmation.type === "purge" ? "danger" : "primary"}
                onClick={() => {
                  void trash.act(confirmation.type, confirmation.ids);
                  setConfirmation(null);
                }}
              >
                {t("settings.history.trash.confirm")}
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
