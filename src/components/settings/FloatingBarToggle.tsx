import React from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { SettingContainer } from "../ui/SettingContainer";
import { useSettings } from "../../hooks/useSettings";
import { commands } from "@/bindings";

interface FloatingBarToggleProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const FloatingBarToggle: React.FC<FloatingBarToggleProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();
    const enabled = getSetting("floating_bar_enabled") ?? false;

    return (
      <SettingContainer
        title={t("settings.advanced.floatingBar.title")}
        description={t("settings.advanced.floatingBar.description")}
        descriptionMode={descriptionMode}
        grouped={grouped}
      >
        <ToggleSwitch
          checked={enabled}
          label={t("settings.advanced.floatingBar.title")}
          description={t("settings.advanced.floatingBar.description")}
          onChange={(checked) => {
            updateSetting("floating_bar_enabled", checked);
            // Window follows the setting live: show creates+raises, hide stows.
            if (checked) {
              void commands.showFloatingBar();
            } else {
              void commands.hideFloatingBar();
            }
          }}
          disabled={isUpdating("floating_bar_enabled")}
        />
      </SettingContainer>
    );
  },
);
