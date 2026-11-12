import { useCallback, useEffect, useState } from "react";
import { commands, type TrashPage } from "@/bindings";

const PAGE_SIZE = 20;
export function useTrash() {
  const [page, setPage] = useState(0);
  const [data, setData] = useState<TrashPage>({ entries: [], total: 0 });
  const [selected, setSelected] = useState<string[]>([]);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [restored, setRestored] = useState(false);
  const load = useCallback(async () => {
    setLoading(true);
    try {
      const result = await commands.listTrashPage(PAGE_SIZE, page * PAGE_SIZE);
      if (result.status === "error") throw new Error(result.error);
      if (page > 0 && result.data.entries.length === 0) {
        setPage(page - 1);
        return;
      }
      setData(result.data);
      setError("");
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, [page]);
  useEffect(() => {
    void load();
  }, [load]);
  const changePage = (next: number) => {
    setSelected([]);
    setPage(next);
  };
  const toggle = (id: string) =>
    setSelected((current) =>
      current.includes(id)
        ? current.filter((item) => item !== id)
        : [...current, id],
    );
  const act = async (action: "restore" | "purge", ids: string[]) => {
    setBusy(true);
    const failures: string[] = [];
    const succeeded: string[] = [];
    for (const id of ids) {
      try {
        const result =
          action === "restore"
            ? await commands.restoreCaptureEntry(id)
            : await commands.purgeTrashedEntry(id);
        if (result.status === "error") throw new Error(result.error);
        if (action === "restore" && result.data === false)
          throw new Error("Entry was not restored");
        succeeded.push(id);
      } catch (err) {
        failures.push(`${id}: ${String(err)}`);
      }
    }
    if (action === "restore" && succeeded.length) setRestored(true);
    setSelected((current) => current.filter((id) => !succeeded.includes(id)));
    await load();
    if (failures.length) setError(failures.join("\n"));
    setBusy(false);
  };
  return {
    page,
    data,
    selected,
    error,
    loading,
    busy,
    restored,
    changePage,
    toggle,
    act,
    load,
    pageSize: PAGE_SIZE,
  };
}
