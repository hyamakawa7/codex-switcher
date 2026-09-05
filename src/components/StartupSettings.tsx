import { useEffect, useRef, useState } from "react";
import { invokeBackend, isTauriRuntime } from "../lib/platform";

interface StartupSettingsState {
  launch_at_login: boolean;
  start_hidden: boolean;
}

export function StartupSettings() {
  const [settings, setSettings] = useState<StartupSettingsState | null>(null);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const savingRef = useRef(false);

  useEffect(() => {
    if (!isTauriRuntime()) return;
    let active = true;
    void invokeBackend<StartupSettingsState | null>("get_startup_settings")
      .then((value) => {
        if (active) setSettings(value);
      })
      .catch((cause: unknown) => {
        if (active) setError(`Could not load startup settings: ${String(cause)}`);
      });
    return () => {
      active = false;
    };
  }, []);

  async function save(key: keyof StartupSettingsState, enabled: boolean) {
    if (!settings || savingRef.current) return;
    savingRef.current = true;
    setSaving(true);
    setError(null);
    try {
      const saved = await invokeBackend<boolean>(
        key === "launch_at_login" ? "set_launch_at_login" : "set_start_hidden",
        { enabled }
      );
      setSettings((previous) => previous && { ...previous, [key]: saved });
    } catch (cause) {
      setError(`Could not save startup settings: ${String(cause)}`);
    } finally {
      savingRef.current = false;
      setSaving(false);
    }
  }

  if (!isTauriRuntime() || (!settings && !error)) return null;

  return (
    <div className="mt-1 border-t border-gray-200 pt-1 dark:border-neutral-800" aria-busy={saving}>
      {settings && (
        <>
          <label className="flex w-full cursor-pointer items-center justify-between gap-2 rounded-lg px-3 py-2 text-sm hover:bg-gray-100 dark:hover:bg-neutral-900">
            <span>Launch at login</span>
            <input
              type="checkbox"
              checked={settings.launch_at_login}
              disabled={saving}
              onChange={(event) => void save("launch_at_login", event.target.checked)}
              className="h-4 w-4 accent-emerald-600 disabled:opacity-50"
            />
          </label>
          <label className="flex w-full cursor-pointer items-center justify-between gap-2 rounded-lg px-3 py-2 text-sm hover:bg-gray-100 dark:hover:bg-neutral-900">
            <span>Start hidden in tray</span>
            <input
              type="checkbox"
              checked={settings.start_hidden}
              disabled={saving}
              onChange={(event) => void save("start_hidden", event.target.checked)}
              className="h-4 w-4 accent-emerald-600 disabled:opacity-50"
            />
          </label>
        </>
      )}
      {error && <p role="alert" className="break-words px-3 py-2 text-xs text-red-600 dark:text-red-400">{error}</p>}
    </div>
  );
}
