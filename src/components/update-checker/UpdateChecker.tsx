import React, { useState, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { getVersion } from "@tauri-apps/api/app";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import { arch, platform } from "@tauri-apps/plugin-os";
import { ProgressBar } from "../shared";
import { useSettings } from "../../hooks/useSettings";
import { commands } from "../../bindings";
import {
  resolvePortableInstallerUrl,
  PORTABLE_RELEASES_URL,
} from "./portableInstaller";

interface UpdateCheckerProps {
  className?: string;
}

// The updater endpoint resolves to this repo's public releases. A build made
// without VITE_UPDATE_SOURCE=public (e.g. local dev) must never call the
// updater against a source it wasn't built to trust, so the footer routes
// to the GitHub releases page instead.
const PRIVATE_RELEASE_ENDPOINT =
  "https://github.com/elmokirk/handy-flow/releases";

const isInAppUpdaterDisabled = () =>
  import.meta.env.VITE_UPDATE_SOURCE !== "public";

// Latest-version probe for the browser-fallback footer link: reads the signed
// updater manifest from the public repo (no auth needed) so the footer can
// ping "new version available" even though in-app download is gated off.
const LATEST_MANIFEST_URL =
  "https://github.com/elmokirk/handy-flow/releases/latest/download/latest.json";

const fetchLatestManifestVersion = async (): Promise<string | null> => {
  try {
    const res = await fetch(LATEST_MANIFEST_URL, { cache: "no-store" });
    if (!res.ok) return null;
    const manifest = (await res.json()) as { version?: string };
    return manifest.version ?? null;
  } catch {
    return null;
  }
};

const UpdateChecker: React.FC<UpdateCheckerProps> = ({ className = "" }) => {
  const { t } = useTranslation();
  // Update checking state
  const [isChecking, setIsChecking] = useState(false);
  const [updateAvailable, setUpdateAvailable] = useState(false);
  const [isInstalling, setIsInstalling] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [showUpToDate, setShowUpToDate] = useState(false);
  const [showPortableUpdateDialog, setShowPortableUpdateDialog] =
    useState(false);
  const [portableInstallerUrl, setPortableInstallerUrl] = useState<string>(
    PORTABLE_RELEASES_URL,
  );
  // Browser-fallback ping: version found in the public latest.json that is
  // newer than the running app. Drives the footer badge next to the
  // "Update via GitHub" link.
  const [fallbackUpdateAvailable, setFallbackUpdateAvailable] =
    useState(false);

  const { settings, isLoading } = useSettings();
  const settingsLoaded = !isLoading && settings !== null;
  const updateChecksEnabled =
    (settings?.update_checks_enabled ?? false) && !isInAppUpdaterDisabled();

  const upToDateTimeoutRef = useRef<ReturnType<typeof setTimeout>>();
  const isManualCheckRef = useRef(false);
  const downloadedBytesRef = useRef(0);
  const contentLengthRef = useRef(0);

  useEffect(() => {
    // Wait for settings to load before doing anything
    if (!settingsLoaded) return;

    if (!updateChecksEnabled) {
      if (upToDateTimeoutRef.current) {
        clearTimeout(upToDateTimeoutRef.current);
      }
      setIsChecking(false);
      setUpdateAvailable(false);
      setShowUpToDate(false);
      return;
    }

    checkForUpdates();
  }, [settingsLoaded, updateChecksEnabled]);

  useEffect(() => {
    // Manual checks are only offered when the in-app updater can actually
    // reach its update source.
    if (isInAppUpdaterDisabled()) return;

    // Listen for update check events
    const updateUnlisten = listen("check-for-updates", () => {
      handleManualUpdateCheck();
    });

    return () => {
      updateUnlisten.then((fn) => fn());
    };
  }, []);

  // Browser-fallback ping: when the in-app updater is gated off, probe the
  // public latest.json once on mount (and on manual clicks) so the footer can
  // still surface that a new release exists.
  useEffect(() => {
    if (!isInAppUpdaterDisabled()) return;
    const probe = async () => {
      const [latest, current] = await Promise.all([
        fetchLatestManifestVersion(),
        getVersion(),
      ]);
      if (latest && current && latest !== current) {
        setFallbackUpdateAvailable(true);
      }
    };
    probe();
  }, []);

  const handleFallbackCheck = async () => {
    const [latest, current] = await Promise.all([
      fetchLatestManifestVersion(),
      getVersion(),
    ]);
    if (latest && current) {
      setFallbackUpdateAvailable(latest !== current);
    }
    openUrl(PRIVATE_RELEASE_ENDPOINT);
  };

  // Update checking functions
  const checkForUpdates = async () => {
    if (!updateChecksEnabled || isChecking) return;

    try {
      setIsChecking(true);
      const update = await check();

      if (update) {
        setUpdateAvailable(true);
        setShowUpToDate(false);
        // Portable installs can't self-update in place — the manual dialog links
        // straight at the matching installer from this manifest instead.
        setPortableInstallerUrl(
          resolvePortableInstallerUrl(update.rawJson, platform(), arch()),
        );
      } else {
        setUpdateAvailable(false);

        if (isManualCheckRef.current) {
          setShowUpToDate(true);
          if (upToDateTimeoutRef.current) {
            clearTimeout(upToDateTimeoutRef.current);
          }
          upToDateTimeoutRef.current = setTimeout(() => {
            setShowUpToDate(false);
          }, 3000);
        }
      }
    } catch (error) {
      console.error("Failed to check for updates:", error);
    } finally {
      setIsChecking(false);
      isManualCheckRef.current = false;
    }
  };

  const handleManualUpdateCheck = () => {
    if (!updateChecksEnabled) return;
    isManualCheckRef.current = true;
    checkForUpdates();
  };

  const installUpdate = async () => {
    if (!updateChecksEnabled) return;

    const portable = await commands.isPortable();
    if (portable) {
      setShowPortableUpdateDialog(true);
      return;
    }
    try {
      setIsInstalling(true);
      setDownloadProgress(0);
      downloadedBytesRef.current = 0;
      contentLengthRef.current = 0;
      const update = await check();

      if (!update) {
        console.log("No update available during install attempt");
        return;
      }

      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            downloadedBytesRef.current = 0;
            contentLengthRef.current = event.data.contentLength ?? 0;
            break;
          case "Progress":
            downloadedBytesRef.current += event.data.chunkLength;
            const progress =
              contentLengthRef.current > 0
                ? Math.round(
                    (downloadedBytesRef.current / contentLengthRef.current) *
                      100,
                  )
                : 0;
            setDownloadProgress(Math.min(progress, 100));
            break;
        }
      });
      await relaunch();
    } catch (error) {
      console.error("Failed to install update:", error);
    } finally {
      setIsInstalling(false);
      setDownloadProgress(0);
      downloadedBytesRef.current = 0;
      contentLengthRef.current = 0;
    }
  };

  // Update status functions
  const getUpdateStatusText = () => {
    if (!updateChecksEnabled) {
      return isInAppUpdaterDisabled()
        ? t("footer.updateViaGitHub")
        : t("footer.updateCheckingDisabled");
    }
    if (isInstalling) {
      return downloadProgress > 0 && downloadProgress < 100
        ? t("footer.downloading", {
            progress: downloadProgress.toString().padStart(3),
          })
        : downloadProgress === 100
          ? t("footer.installing")
          : t("footer.preparing");
    }
    if (isChecking) return t("footer.checkingUpdates");
    if (showUpToDate) return t("footer.upToDate");
    if (updateAvailable) return t("footer.updateAvailableShort");
    return t("footer.checkForUpdates");
  };

  const getUpdateStatusAction = () => {
    if (!updateChecksEnabled) {
      return isInAppUpdaterDisabled() ? handleFallbackCheck : undefined;
    }
    if (updateAvailable && !isInstalling) return installUpdate;
    if (!isChecking && !isInstalling && !updateAvailable)
      return handleManualUpdateCheck;
    return undefined;
  };

  const isUpdateDisabled = !updateChecksEnabled || isChecking || isInstalling;
  const isUpdateClickable =
    !isUpdateDisabled && (updateAvailable || (!isChecking && !showUpToDate));
  // The fallback link is always a real button: clicking it pings latest.json
  // and opens the releases page.
  const isFallbackLink =
    !updateChecksEnabled && isInAppUpdaterDisabled() && !isUpdateClickable;

  // When no installer could be resolved for this target the button falls back to
  // the releases index, so the dialog has to say "browse" rather than "download".
  const hasDirectInstaller = portableInstallerUrl !== PORTABLE_RELEASES_URL;

  return (
    <>
      {showPortableUpdateDialog && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-background border border-mid-gray/20 rounded-lg p-6 max-w-md w-full mx-4 space-y-4">
            <h2 className="text-base font-semibold">
              {t("footer.portableUpdateTitle")}
            </h2>
            <p className="text-sm text-text/70">
              {hasDirectInstaller
                ? t("footer.portableUpdateMessage")
                : t("footer.portableUpdateBrowseMessage")}
            </p>
            <div className="flex gap-2 justify-end">
              <button
                className="px-3 py-1.5 text-sm rounded border border-mid-gray/20 hover:bg-mid-gray/10 transition-colors"
                onClick={() => setShowPortableUpdateDialog(false)}
              >
                {t("common.close")}
              </button>
              <button
                className="px-3 py-1.5 text-sm rounded bg-logo-primary text-white hover:bg-logo-primary/80 transition-colors"
                onClick={() => {
                  openUrl(portableInstallerUrl);
                  setShowPortableUpdateDialog(false);
                }}
              >
                {hasDirectInstaller
                  ? t("footer.portableUpdateButton")
                  : t("footer.portableUpdateBrowseButton")}
              </button>
            </div>
          </div>
        </div>
      )}
      <div className={`flex items-center gap-3 ${className}`}>
        {isUpdateClickable || isFallbackLink ? (
          <button
            onClick={getUpdateStatusAction()}
            disabled={isUpdateDisabled}
            className={`transition-colors disabled:opacity-50 tabular-nums ${
              updateAvailable || (isFallbackLink && fallbackUpdateAvailable)
                ? "text-logo-primary hover:text-logo-primary/80 font-medium"
                : "text-text/60 hover:text-text/80"
            }`}
          >
            {getUpdateStatusText()}
          </button>
        ) : (
          <span className="text-text/60 tabular-nums">
            {getUpdateStatusText()}
          </span>
        )}

        {/* Ping badge: drawn whenever a newer release is known (in-app updater
            or the browser-fallback probe), so an update is visible at a glance
            even when the label itself is a plain state string. */}
        {(updateAvailable || (isFallbackLink && fallbackUpdateAvailable)) && (
          <span
            className="relative flex h-2.5 w-2.5 shrink-0"
            title={t("footer.updateAvailableShort")}
          >
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-logo-primary opacity-60" />
            <span className="relative inline-flex rounded-full h-2.5 w-2.5 bg-logo-primary" />
          </span>
        )}

        {isInstalling && downloadProgress > 0 && downloadProgress < 100 && (
          <ProgressBar
            progress={[
              {
                id: "update",
                percentage: downloadProgress,
              },
            ]}
            size="large"
          />
        )}
      </div>
    </>
  );
};

export default UpdateChecker;
