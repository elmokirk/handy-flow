import { test, expect } from "@playwright/test";

test("trash pagination, selected restore, confirmed purge and empty state", async ({
  page,
}) => {
  await page.addInitScript(() => {
    const entries = Array.from({ length: 21 }, (_, i) => ({
      capture_id: `id-${i}`,
      title: `Entry ${i}`,
      created_at_ms: 1700000000000,
      deleted_at_ms: 1700000000000,
      audio_file_name: null,
      integrity_state: "audio_missing",
    }));
    const calls: string[] = [];
    (window as unknown as { calls: string[] }).calls = calls;
    (
      window as unknown as { __TAURI_INTERNALS__: unknown }
    ).__TAURI_INTERNALS__ = {
      invoke: async (cmd: string, args: Record<string, unknown>) => {
        calls.push(`${cmd}:${String(args?.captureId ?? "")}`);
        if (cmd === "list_trash_page")
          return {
            entries: entries.slice(
              Number(args.offset),
              Number(args.offset) + Number(args.limit),
            ),
            total: entries.length,
          };
        if (cmd === "restore_capture_entry" || cmd === "purge_trashed_entry") {
          const index = entries.findIndex(
            (e) => e.capture_id === args.captureId,
          );
          if (index >= 0) entries.splice(index, 1);
          return cmd === "restore_capture_entry" ? true : null;
        }
        return null;
      },
      transformCallback: (cb: unknown) => cb,
      metadata: { currentWindow: { label: "main" } },
    };
  });
  await page.goto("/tests/trash-fixture.html");
  await page.getByRole("button", { name: "Open trash" }).click();
  await expect(page.getByText("Trash entries: 21")).toBeVisible();
  await page.getByRole("button", { name: "Next" }).click();
  await expect(page.getByText("Entry 20", { exact: true })).toBeVisible();
  await page.getByRole("checkbox", { name: "Select Entry 20" }).check();
  await page.getByRole("button", { name: "Restore selected" }).click();
  await expect(page.getByRole("dialog")).toBeVisible();
  await page.getByRole("button", { name: "Confirm" }).click();
  await expect(page.getByText(/older unsaved entries/)).toBeVisible();
  await expect(page.getByText("Trash entries: 20")).toBeVisible();
  await page
    .getByRole("button", { name: "Permanently delete", exact: true })
    .first()
    .click();
  await page.getByRole("button", { name: "Confirm" }).click();
  await expect(page.getByText("Trash entries: 19")).toBeVisible();
  for (const checkbox of await page.getByRole("checkbox").all())
    await checkbox.check();
  await page
    .getByRole("button", { name: "Permanently delete selected" })
    .click();
  await page.getByRole("button", { name: "Confirm" }).click();
  await expect(page.getByText("The trash is empty.")).toBeVisible();
  const calls = await page.evaluate(
    () => (window as unknown as { calls: string[] }).calls,
  );
  expect(calls).toContain("restore_capture_entry:id-20");
  expect(
    calls.filter((call) => call.startsWith("purge_trashed_entry:")),
  ).toHaveLength(20);
});

test("purge errors remain visible and do not remove the entry", async ({
  page,
}) => {
  await page.addInitScript(() => {
    (
      window as unknown as { __TAURI_INTERNALS__: unknown }
    ).__TAURI_INTERNALS__ = {
      invoke: async (cmd: string) => {
        if (cmd === "list_trash_page")
          return {
            entries: [
              {
                capture_id: "locked",
                title: "Locked entry",
                created_at_ms: 1700000000000,
                deleted_at_ms: null,
                audio_file_name: "a.wav",
                integrity_state: "audio_valid",
              },
            ],
            total: 1,
          };
        if (cmd === "purge_trashed_entry") throw "File locked";
        return null;
      },
      transformCallback: (cb: unknown) => cb,
      metadata: { currentWindow: { label: "main" } },
    };
  });
  await page.goto("/tests/trash-fixture.html");
  await page.getByRole("button", { name: "Open trash" }).click();
  await page
    .getByRole("button", { name: "Permanently delete", exact: true })
    .click();
  await page.getByRole("button", { name: "Confirm" }).click();
  await expect(page.getByRole("alert")).toContainText("File locked");
  await expect(page.getByText("Locked entry", { exact: true })).toBeVisible();
});
