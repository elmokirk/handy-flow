import React from "react";
import { useTranslation } from "react-i18next";
import {
  BookOpen,
  Cog,
  FlaskConical,
  History,
  Info,
  Sparkles,
  Cpu,
  SlidersHorizontal,
} from "lucide-react";
import HandyFlowTextLogo from "./icons/HandyFlowTextLogo";
import VulcanHand from "./icons/VulcanHand";
import { useSettings } from "../hooks/useSettings";
import {
  GeneralSettings,
  AdvancedSettings,
  HistorySettings,
  DebugSettings,
  AboutSettings,
  PostProcessingSettings,
  ModelsSettings,
  KnowledgeSettings,
} from "./settings";

export type SidebarSection = keyof typeof SECTIONS_CONFIG;

interface IconProps {
  width?: number | string;
  height?: number | string;
  size?: number | string;
  className?: string;
  [key: string]: any;
}

interface SectionConfig {
  labelKey: string;
  icon: React.ComponentType<IconProps>;
  component: React.ComponentType;
  enabled: (settings: any) => boolean;
}

/* Primary sections render as labelled rows at the top of the sidebar — the
   areas a dictation user touches daily. Utility sections (models, advanced)
   and app-level entries (debug, about) render as an icon-only strip pinned to
   the bottom, the way settings live in a bottom bar in comparable apps. */
export const SECTIONS_CONFIG = {
  knowledge: {
    labelKey: "sidebar.knowledge",
    icon: BookOpen,
    component: KnowledgeSettings,
    enabled: () => true,
  },
  history: {
    labelKey: "sidebar.history",
    icon: History,
    component: HistorySettings,
    enabled: () => true,
  },
  general: {
    labelKey: "sidebar.general",
    icon: VulcanHand,
    component: GeneralSettings,
    enabled: () => true,
  },
  postprocessing: {
    labelKey: "sidebar.postProcessing",
    icon: Sparkles,
    component: PostProcessingSettings,
    enabled: (settings) => settings?.post_process_enabled ?? false,
  },
  models: {
    labelKey: "sidebar.models",
    icon: Cpu,
    component: ModelsSettings,
    enabled: () => true,
  },
  advanced: {
    labelKey: "sidebar.advanced",
    icon: SlidersHorizontal,
    component: AdvancedSettings,
    enabled: () => true,
  },
  debug: {
    labelKey: "sidebar.debug",
    icon: FlaskConical,
    component: DebugSettings,
    enabled: (settings) => settings?.debug_mode ?? false,
  },
  about: {
    labelKey: "sidebar.about",
    icon: Info,
    component: AboutSettings,
    enabled: () => true,
  },
} as const satisfies Record<string, SectionConfig>;

const PRIMARY_SECTIONS: SidebarSection[] = [
  "knowledge",
  "history",
  "general",
  "postprocessing",
];
const UTILITY_SECTIONS: SidebarSection[] = ["models", "advanced"];
const APP_SECTIONS: SidebarSection[] = ["debug", "about"];

interface SidebarProps {
  activeSection: SidebarSection;
  onSectionChange: (section: SidebarSection) => void;
}

export const Sidebar: React.FC<SidebarProps> = ({
  activeSection,
  onSectionChange,
}) => {
  const { t } = useTranslation();
  const { settings } = useSettings();

  const isAvailable = (id: SidebarSection) =>
    SECTIONS_CONFIG[id].enabled(settings);

  const primarySections = PRIMARY_SECTIONS.filter(isAvailable);
  const utilitySections = UTILITY_SECTIONS.filter(isAvailable);
  const appSections = APP_SECTIONS.filter(isAvailable);

  const renderLabelledSection = (section: SidebarSection) => {
    const config = SECTIONS_CONFIG[section];
    const Icon = config.icon;
    const isActive = activeSection === section;

    return (
      <div
        key={section}
        className={`flex gap-2 items-center p-2 w-full rounded-lg cursor-pointer transition-colors ${
          isActive
            ? "bg-logo-primary/80"
            : "hover:bg-mid-gray/20 hover:opacity-100 opacity-85"
        }`}
        onClick={() => onSectionChange(section)}
      >
        <Icon width={24} height={24} className="shrink-0" />
        <p className="text-sm font-medium truncate" title={t(config.labelKey)}>
          {t(config.labelKey)}
        </p>
      </div>
    );
  };

  const renderIconSection = (section: SidebarSection) => {
    const config = SECTIONS_CONFIG[section];
    const Icon = config.icon;
    const isActive = activeSection === section;

    return (
      <div
        key={section}
        className={`flex items-center justify-center p-2 rounded-lg cursor-pointer transition-colors ${
          isActive
            ? "bg-logo-primary/80"
            : "hover:bg-mid-gray/20 hover:opacity-100 opacity-70"
        }`}
        onClick={() => onSectionChange(section)}
        title={t(config.labelKey)}
        aria-label={t(config.labelKey)}
      >
        <Icon width={20} height={20} className="shrink-0" />
      </div>
    );
  };

  return (
    <div className="flex flex-col w-40 h-full border-e border-mid-gray/20 items-center px-2">
      <HandyFlowTextLogo width={140} className="m-4" />
      <div className="flex flex-col w-full items-center gap-1 pt-2 border-t border-mid-gray/20">
        {primarySections.map(renderLabelledSection)}
      </div>
      <div className="mt-auto w-full flex flex-col items-center pt-2 border-t border-mid-gray/20 pb-3 gap-1">
        <div className="flex flex-row w-full items-center justify-center gap-1">
          {utilitySections.map(renderIconSection)}
        </div>
        <div className="flex flex-row w-full items-center justify-center gap-1">
          {appSections.map(renderIconSection)}
        </div>
      </div>
    </div>
  );
};
