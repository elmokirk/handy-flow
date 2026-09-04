import { test, expect, type Page } from "@playwright/test";

/**
 * NOTE-304 acceptance (UI half): note search and restore in the scratchpad.
 *
 * Plain Playwright has no Tauri backend, so `window.__TAURI_INTERNALS__` is
 * stubbed with an in-memory note store before the bundle boots. That keeps
 * the REAL component, hook and debounce logic under test — only the IPC
 * boundary is faked. The search semantics themselves (FTS, bounding,
 * pinned-first ordering, append-only restore) are covered natively in
 * `src-tauri/tests/note_search/note_search_test.rs`.
 */

const SCRATCHPAD_URL = "/src/scratchpad/index.html";

/** Install a fake Tauri IPC bridge backed by a small note fixture. */
async function mockTauri(page: Page) {
  await page.addInitScript(() => {
    const notes = [
      {
        id: "n1",
        title: "Rust Notizen",
        pinned: false,
        trashed: false,
        created_at_ms: 1,
        updated_at_ms: 30,
        body: "ownership und borrowing",
      },
      {
        id: "n2",
        title: "Einkauf",
        pinned: true,
        trashed: false,
        created_at_ms: 2,
        updated_at_ms: 20,
        body: "milch brot butter",
      },
      {
        id: "n3",
        title: "Reise",
        pinned: false,
        trashed: false,
        created_at_ms: 3,
        updated_at_ms: 10,
        body: "ownership der route planen",
      },
    ];
    const dto = (n: (typeof notes)[number]) => ({
      id: n.id,
      title: n.title,
      pinned: n.pinned,
      trashed: n.trashed,
      created_at_ms: n.created_at_ms,
      updated_at_ms: n.updated_at_ms,
    });
    const calls: Array<{ cmd: string; args: unknown }> = [];
    (window as unknown as { __CALLS__: typeof calls }).__CALLS__ = calls;

    const invoke = async (cmd: string, args: Record<string, unknown> = {}) => {
      calls.push({ cmd, args });
      switch (cmd) {
        case "notes_list":
          // Pinned first, then newest — mirrors list_active_notes.
          return [...notes]
            .sort(
              (a, b) =>
                Number(b.pinned) - Number(a.pinned) ||
                b.updated_at_ms - a.updated_at_ms,
            )
            .map(dto);
        case "notes_search": {
          const q = String(args.query ?? "")
            .trim()
            .toLowerCase();
          if (!q) return [];
          const limit = Number(args.limit ?? 20);
          return notes
            .filter(
              (n) =>
                n.title.toLowerCase().includes(q) ||
                n.body.toLowerCase().includes(q),
            )
            .sort((a, b) => Number(b.pinned) - Number(a.pinned))
            .slice(0, limit)
            .map(dto);
        }
        case "notes_current": {
          const n = notes.find((x) => x.id === args.noteId);
          return n
            ? {
                id: `${n.id}-v1`,
                note_id: n.id,
                version_no: 1,
                content: n.body,
                source: "manual_edit",
                created_at_ms: n.created_at_ms,
              }
            : null;
        }
        case "notes_append":
          return null; // hash dedupe: unchanged content creates no version
        case "notes_versions":
          return [];
        case "prompt_profiles_list":
          return [];
        default:
          return null;
      }
    };

    (
      window as unknown as { __TAURI_INTERNALS__: Record<string, unknown> }
    ).__TAURI_INTERNALS__ = {
      invoke,
      transformCallback: (cb: unknown) => cb,
      metadata: { currentWindow: { label: "scratchpad" } },
    };
  });
}

test.describe("Notes search (NOTE-304)", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauri(page);
    await page.goto(SCRATCHPAD_URL);
    await expect(page.getByTestId("scratchpad-root")).toBeVisible();
  });

  test("search panel is closed until toggled", async ({ page }) => {
    await expect(page.getByTestId("scratchpad-search")).toBeHidden();
    await page.getByTestId("scratchpad-search-toggle").click();
    await expect(page.getByTestId("scratchpad-search-input")).toBeVisible();
  });

  test("an empty query lists nothing instead of every note", async ({
    page,
  }) => {
    await page.getByTestId("scratchpad-search-toggle").click();
    await expect(page.getByTestId("scratchpad-search-results")).toBeHidden();

    // Whitespace is still an empty query.
    await page.getByTestId("scratchpad-search-input").fill("   ");
    await expect(page.getByTestId("scratchpad-search-results")).toBeHidden();

    const searched = await page.evaluate(() =>
      (
        window as unknown as { __CALLS__: Array<{ cmd: string }> }
      ).__CALLS__.some((c) => c.cmd === "notes_search"),
    );
    expect(searched, "no backend call for an empty query").toBe(false);
  });

  test("matching notes are listed with pinned first", async ({ page }) => {
    await page.getByTestId("scratchpad-search-toggle").click();
    await page.getByTestId("scratchpad-search-input").fill("ownership");

    const items = page
      .getByTestId("scratchpad-search-results")
      .getByRole("button");
    await expect(items).toHaveCount(2);
    await expect(items.nth(0)).toContainText("Rust Notizen");
    await expect(items.nth(1)).toContainText("Reise");

    // A term hitting the pinned note puts it on top.
    await page.getByTestId("scratchpad-search-input").fill("e");
    await expect(
      page.getByTestId("scratchpad-search-results").getByRole("button").first(),
    ).toContainText("Einkauf");
  });

  test("a query with no hits shows the empty message", async ({ page }) => {
    await page.getByTestId("scratchpad-search-toggle").click();
    await page.getByTestId("scratchpad-search-input").fill("quantenphysik");
    await expect(page.getByTestId("scratchpad-search-results")).toContainText(
      "No matches.",
    );
  });

  test("the search is debounced into one backend call", async ({ page }) => {
    await page.getByTestId("scratchpad-search-toggle").click();
    const input = page.getByTestId("scratchpad-search-input");
    // Type progressively; only the settled term should reach the backend.
    for (const term of ["o", "ow", "own", "owne", "owner"]) {
      await input.fill(term);
    }
    await expect(
      page.getByTestId("scratchpad-search-results").getByRole("button"),
    ).toHaveCount(2);

    const queries = await page.evaluate(() =>
      (
        window as unknown as {
          __CALLS__: Array<{ cmd: string; args: { query?: string } }>;
        }
      ).__CALLS__
        .filter((c) => c.cmd === "notes_search")
        .map((c) => c.args.query),
    );
    expect(queries).toEqual(["owner"]);
  });

  test("search requests stay bounded", async ({ page }) => {
    await page.getByTestId("scratchpad-search-toggle").click();
    await page.getByTestId("scratchpad-search-input").fill("ownership");
    await expect(
      page.getByTestId("scratchpad-search-results").getByRole("button"),
    ).toHaveCount(2);

    const limits = await page.evaluate(() =>
      (
        window as unknown as {
          __CALLS__: Array<{ cmd: string; args: { limit?: number } }>;
        }
      ).__CALLS__
        .filter((c) => c.cmd === "notes_search")
        .map((c) => c.args.limit),
    );
    expect(limits.length).toBeGreaterThan(0);
    for (const l of limits) {
      expect(l).toBeGreaterThan(0);
      expect(l).toBeLessThanOrEqual(200);
    }
  });

  test("opening a result loads that note and closes the panel", async ({
    page,
  }) => {
    // The pad opens the pinned note first (notes_list ordering).
    await expect(page.getByTestId("scratchpad-editor")).toHaveValue(
      "milch brot butter",
    );

    await page.getByTestId("scratchpad-search-toggle").click();
    await page.getByTestId("scratchpad-search-input").fill("borrowing");
    await page
      .getByTestId("scratchpad-search-results")
      .getByRole("button")
      .first()
      .click();

    await expect(page.getByTestId("scratchpad-editor")).toHaveValue(
      "ownership und borrowing",
    );
    await expect(page.getByTestId("scratchpad-search")).toBeHidden();
  });
});
