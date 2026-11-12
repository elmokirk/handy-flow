import React from "react";
import { createRoot } from "react-dom/client";
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import translation from "../src/i18n/locales/en/translation.json";
import { TrashView } from "../src/components/settings/history/TrashView";
import { useSettingsStore } from "../src/stores/settingsStore";

void i18n.use(initReactI18next).init({
  lng: "en",
  resources: { en: { translation } },
  interpolation: { escapeValue: false },
});
useSettingsStore.setState({
  isLoading: false,
  settings: { recording_retention_period: "preserve_limit" },
});
function Fixture() {
  const [open, setOpen] = React.useState(false);
  return open ? (
    <TrashView onClose={() => setOpen(false)} />
  ) : (
    <button onClick={() => setOpen(true)}>Open trash</button>
  );
}
createRoot(document.getElementById("root")!).render(<Fixture />);
