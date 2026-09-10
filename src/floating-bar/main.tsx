import React, { useEffect, useState } from "react";
import ReactDOM from "react-dom/client";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { commands, events } from "@/bindings";
import { VulcanHand } from "./VulcanHand";
import "@/i18n";

const BRAND_BG = "#0a0a0a";
const ORANGE = "#fb923c";
const WHITE = "rgba(255, 255, 255, 0.95)";

type BarState = "idle" | "recording" | "working";

const FloatingBar: React.FC = () => {
  const [state, setState] = useState<BarState>("idle");

  useEffect(() => {
    // Mirror the recording state the same way the tray does: is_recording on
    // mount, then phase events while the bar is alive.
    void commands.isRecording().then((recording) => {
      if (recording) setState("recording");
    });

    const unlisteners: Array<() => void> = [];
    void (async () => {
      unlisteners.push(
        await listen<boolean>("recording-state", (e) =>
          setState(e.payload ? "recording" : "idle"),
        ),
      );
      unlisteners.push(
        await listen<string>("bar-work-state", (e) => {
          setState(e.payload === "working" ? "working" : "idle");
        }),
      );
    })();

    return () => unlisteners.forEach((fn) => fn());
  }, []);

  const toggle = async () => {
    if (state === "working") {
      await commands.cancelOperation();
      return;
    }
    // Same entry point the global shortcut and tray use — one code path for
    // toggle semantics (start when idle, stop when recording).
    await commands.toggleTranscription();
  };

  const label =
    state === "idle" ? "Start" : state === "recording" ? "Stop" : "…";

  // Click = toggle dictation; press-and-move = drag (the threshold prevents
  // the drag gesture from swallowing plain clicks).
  const onPointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (e.button !== 0) return;
    const startX = e.clientX;
    const startY = e.clientY;
    let dragged = false;
    const el = e.currentTarget;
    el.setPointerCapture(e.pointerId);
    const onMove = (ev: PointerEvent) => {
      if (
        !dragged &&
        Math.hypot(ev.clientX - startX, ev.clientY - startY) > 4
      ) {
        dragged = true;
        void getCurrentWindow().startDragging();
      }
    };
    const onUp = (ev: PointerEvent) => {
      el.releasePointerCapture(e.pointerId);
      el.removeEventListener("pointermove", onMove);
      el.removeEventListener("pointerup", onUp);
      if (!dragged && ev.button === 0) {
        void toggle();
      }
    };
    el.addEventListener("pointermove", onMove);
    el.addEventListener("pointerup", onUp);
  };

  return (
    <div
      onPointerDown={onPointerDown}
      style={{
        width: "100%",
        height: "100%",
        boxSizing: "border-box",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        gap: 7,
        background: BRAND_BG,
        border: `1px solid rgba(255, 255, 255, 0.95)`,
        borderRadius: 9999,
        cursor: "pointer",
        userSelect: "none",
        WebkitUserSelect: "none",
        fontFamily:
          '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
        transition: "transform 100ms ease",
      }}
      onMouseDown={(e) => {
        (e.currentTarget as HTMLDivElement).style.transform = "scale(0.97)";
      }}
      onMouseUp={(e) => {
        (e.currentTarget as HTMLDivElement).style.transform = "scale(1)";
      }}
      title="Handy Flow — click to toggle, drag to move"
    >
      <VulcanHand
        width={16}
        height={16}
        color={state === "idle" ? ORANGE : "#ef4444"}
      />
      <span
        style={{
          color: ORANGE,
          fontSize: 12,
          fontWeight: 600,
          letterSpacing: "0.01em",
          lineHeight: 1,
          whiteSpace: "nowrap",
        }}
      >
        {state === "working" ? (
          <span
            style={{
              display: "inline-block",
              width: 10,
              height: 10,
              borderRadius: "50%",
              border: `2px solid ${WHITE}`,
              borderTopColor: ORANGE,
              animation: "fbspin 0.7s linear infinite",
            }}
          />
        ) : (
          label
        )}
      </span>
      <style>{`@keyframes fbspin { to { transform: rotate(1turn); } }`}</style>
    </div>
  );
};

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <FloatingBar />
  </React.StrictMode>,
);
