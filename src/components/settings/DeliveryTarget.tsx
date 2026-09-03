import React from "react";
import { useTranslation } from "react-i18next";
import { Dropdown, type DropdownOption } from "../ui/Dropdown";
import { SettingContainer } from "../ui/SettingContainer";
import { useSettings } from "../../hooks/useSettings";
import type { DeliveryTarget } from "@/bindings";

interface DeliveryTargetProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

/**
 * Where a finished transcript goes (PAD-305). Each option maps to one
 * backend `DeliverySink`; the paste method below only governs *how* the
 * focused-app option types.
 */
export const DeliveryTargetSetting: React.FC<DeliveryTargetProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();

    const selected = (getSetting("delivery_target") ||
      "focused_app") as DeliveryTarget;

    const options: DropdownOption[] = [
      {
        value: "focused_app",
        label: t("settings.advanced.deliveryTarget.options.focusedApp"),
      },
      {
        value: "scratchpad",
        label: t("settings.advanced.deliveryTarget.options.scratchpad"),
      },
      {
        value: "clipboard",
        label: t("settings.advanced.deliveryTarget.options.clipboard"),
      },
    ];

    return (
      <SettingContainer
        title={t("settings.advanced.deliveryTarget.title")}
        description={t("settings.advanced.deliveryTarget.description")}
        descriptionMode={descriptionMode}
        grouped={grouped}
        tooltipPosition="bottom"
      >
        <Dropdown
          options={options}
          selectedValue={selected}
          onSelect={(value) =>
            updateSetting("delivery_target", value as DeliveryTarget)
          }
          disabled={isUpdating("delivery_target")}
        />
      </SettingContainer>
    );
  },
);
