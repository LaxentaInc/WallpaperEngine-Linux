import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  // const [name, setName] = useState("");

  // async function greet() {
  //   // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  //   setGreetMsg(await invoke("greet", { name }));
  // }

  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>

      <div className="row">
        <a href="https://vite.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <div className="row" style={{ marginTop: "2rem" }}>
        <button 
          onClick={async () => {
            try {
              await invoke("cmd_set_video_wallpaper", { 
                videoPath: "/home/laxenta/wallpaperengine-local/BlueArchive.mp4",
                monitorId: null
              });
              setGreetMsg("Wallpaper spawn command sent!");
            } catch (e) {
              setGreetMsg("Error: " + e);
            }
          }}
          style={{ padding: "10px 20px", fontSize: "1.2rem", cursor: "pointer" }}
        >
          Launch Video Wallpaper
        </button>
      </div>
      <p style={{ marginTop: "1rem" }}>{greetMsg}</p>
    </main>
  );
}

export default App;
