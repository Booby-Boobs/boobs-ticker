import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";

interface TickerData {
  soul: number;
  news: string[];
}

function App() {
  const [soul, setSoul] = useState(100.0); // Will be updated from backend
  const [news, setNews] = useState<string[]>([]);
  const boostEnergy = async () => {
    await invoke("boost_energy");
  };

  const annoyEnergy = async () => {
    await invoke("annoy_energy");
  };

  useEffect(() => {
    const unlisten = listen<TickerData>("ticker-update", (event) => {
      setSoul(event.payload.soul);
      if (event.payload.news.length > 0 && news.length === 0) {
        setNews(event.payload.news);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [news.length]);



  return (
    <div className="h-36 bg-gray-100 text-black p-2">
        <div className="flex items-center justify-between h-8">
           <div className={`text-xs font-bold font-['Press_Start_2P'] ${soul > 70 ? "text-green-500" : soul > 50 ? "text-orange-500" : soul > 20 ? "text-yellow-500" : soul > 0 ? "text-red-500" : "text-red-700"}`}>
             HP: {soul.toFixed(0)}%
           </div>
         <div className="flex-1 mx-4 overflow-hidden">
           <div className="whitespace-nowrap animate-marquee font-['Press_Start_2P'] text-sm">
             {news.length > 0 && (
               <span>{news.join(" | ")}</span>
             )}
           </div>
         </div>
        <button
          onClick={boostEnergy}
          className="px-2 py-1 bg-pink-500 hover:bg-pink-600 rounded text-sm text-white mr-2"
        >
          Endure +5%
        </button>
         <button
           onClick={annoyEnergy}
           className="px-2 py-1 bg-gray-300 hover:bg-gray-400 rounded text-sm text-black"
         >
           Annoyed -5%
         </button>

        </div>
        <div className="h-4 bg-gray-300 rounded overflow-hidden mt-1">
          <div
            className={`h-full transition-all duration-300 ${soul > 70 ? "bg-green-500" : soul > 50 ? "bg-orange-500" : soul > 20 ? "bg-yellow-500" : "bg-red-500"}`}
            style={{ width: `${Math.max(soul, 0)}%` }}
          ></div>
        </div>

      {soul <= 0 && (
        <div className="text-center text-xs text-red-600 mt-1">
          You've been working too hard. Please visit here for your medical certificate.
          <br />
          <a href="#" onClick={() => openUrl("https://booby.dev/medical-certificate")} className="text-blue-500 underline">Medical Certificate Page</a>
        </div>
      )}

      <div className="text-center text-xs text-gray-500 mt-1">
        <img src="/everyday.png" className="w-4 h-4 rounded-full inline mr-1" />
        we are <a href="#" onClick={() => openUrl("https://booby.dev/about")}><img src="/boobs.png" className="h-5 w-auto inline" /></a>
      </div>
    </div>
  );
}

export default App;
