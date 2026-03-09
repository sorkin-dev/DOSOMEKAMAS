import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface AppInfo {
  name: string;
  version: string;
  description: string;
}

function App(): React.JSX.Element {
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);

  useEffect(() => {
    invoke<AppInfo>("get_app_info")
      .then((info) => setAppInfo(info))
      .catch((err: unknown) => {
        console.error("Failed to load app info:", err);
      });
  }, []);

  return (
    <main className="app-container">
      <div className="app-header">
        <h1 className="app-title">DOSOMEKAMAS</h1>
        <p className="app-tagline">Assistant Forgemagie — Dofus 3.5</p>
      </div>
      {appInfo !== null && (
        <div className="app-info">
          <span className="app-version">v{appInfo.version}</span>
        </div>
      )}
    </main>
  );
}

export default App;
