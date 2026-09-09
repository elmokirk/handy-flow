import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { SettingContainer } from "../../ui/SettingContainer";
import { Button } from "../../ui/Button";
import { AppDataDirectory } from "../AppDataDirectory";
import { AppLanguageSelector } from "../AppLanguageSelector";
import { ShowWhatsNewOnUpdate } from "../ShowWhatsNewOnUpdate";
import { ThemeSelector } from "../ThemeSelector";
import { LogDirectory } from "../debug";
import VulcanHand from "../../icons/VulcanHand";

const RELEASES_URL = "https://github.com/elmokirk/handy-flow/releases";
const SOURCE_URL = "https://github.com/elmokirk/handy-flow";
const UPSTREAM_URL = "https://github.com/cjpais/Handy";
const UPSTREAM_SPONSOR_URL = "https://github.com/sponsors/cjpais";

export const AboutSettings: React.FC = () => {
  const { t } = useTranslation();
  const [version, setVersion] = useState("");

  useEffect(() => {
    const fetchVersion = async () => {
      try {
        const appVersion = await getVersion();
        setVersion(appVersion);
      } catch (error) {
        console.error("Failed to get app version:", error);
        setVersion("0.0.0");
      }
    };

    fetchVersion();
  }, []);

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <div className="w-full border border-logo-primary/40 bg-logo-primary/10 rounded-lg p-4 flex items-start gap-3">
        <div className="shrink-0 mt-0.5">
          <VulcanHand width={28} height={28} />
        </div>
        <div className="flex flex-col gap-1">
          <p className="text-sm font-semibold">
            {t("settings.about.forkNotice.title")}
          </p>
          <p className="text-sm text-text/70">
            {t("settings.about.forkNotice.description")}
          </p>
          <div className="flex gap-2 pt-1">
            <Button
              variant="secondary"
              size="sm"
              onClick={() => openUrl(UPSTREAM_URL)}
            >
              {t("settings.about.forkNotice.upstreamButton")}
            </Button>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => openUrl(UPSTREAM_SPONSOR_URL)}
            >
              {t("settings.about.forkNotice.sponsorButton")}
            </Button>
          </div>
        </div>
      </div>

      <SettingsGroup title={t("settings.about.title")}>
        <AppLanguageSelector descriptionMode="tooltip" grouped={true} />
        <ThemeSelector descriptionMode="tooltip" grouped={true} />
        <SettingContainer
          title={t("settings.about.version.title")}
          description={t("settings.about.version.description")}
          grouped={true}
        >
          {/* eslint-disable-next-line i18next/no-literal-string */}
          <span className="text-sm font-mono">v{version}</span>
        </SettingContainer>

        <ShowWhatsNewOnUpdate descriptionMode="tooltip" grouped={true} />

        <SettingContainer
          title={t("settings.about.releases.title")}
          description={t("settings.about.releases.description")}
          grouped={true}
        >
          <Button
            variant="primary"
            size="md"
            onClick={() => openUrl(RELEASES_URL)}
          >
            {t("settings.about.releases.button")}
          </Button>
        </SettingContainer>

        <SettingContainer
          title={t("settings.about.sourceCode.title")}
          description={t("settings.about.sourceCode.description")}
          grouped={true}
        >
          <Button
            variant="secondary"
            size="md"
            onClick={() => openUrl(SOURCE_URL)}
          >
            {t("settings.about.sourceCode.button")}
          </Button>
        </SettingContainer>

        <AppDataDirectory descriptionMode="tooltip" grouped={true} />
        <LogDirectory grouped={true} />
      </SettingsGroup>

      <SettingsGroup title={t("settings.about.acknowledgments.title")}>
        <SettingContainer
          title={t("settings.about.acknowledgments.ggml.title")}
          description={t("settings.about.acknowledgments.ggml.description")}
          grouped={true}
          layout="stacked"
        >
          <div className="text-sm text-mid-gray">
            {t("settings.about.acknowledgments.ggml.details")}
          </div>
        </SettingContainer>
        <SettingContainer
          title={t("settings.about.acknowledgments.upstream.title")}
          description={t("settings.about.acknowledgments.upstream.description")}
          grouped={true}
          layout="stacked"
        >
          <div className="text-sm text-mid-gray">
            {t("settings.about.acknowledgments.upstream.details")}
          </div>
        </SettingContainer>
        <SettingContainer
          title={t("settings.about.acknowledgments.sponsor.title")}
          description={t("settings.about.acknowledgments.sponsor.description")}
          grouped={true}
          layout="stacked"
        >
          <Button
            variant="secondary"
            size="md"
            onClick={() => openUrl(UPSTREAM_SPONSOR_URL)}
          >
            {t("settings.about.acknowledgments.sponsor.button")}
          </Button>
        </SettingContainer>
      </SettingsGroup>
    </div>
  );
};
