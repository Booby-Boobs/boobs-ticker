import { useState, useEffect, memo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import { checkAccessibilityPermission, checkInputMonitoringPermission } from 'tauri-plugin-macos-permissions-api';
import { ask } from '@tauri-apps/plugin-dialog';
import { open } from '@tauri-apps/plugin-shell';

interface TickerData {
  soul: number;
  total_time: number;
  keys_total: number;
  clicks_total: number;
  mouse_total: number;
}



const Ticker = memo(({ text, duration }: { text: string, duration: number }) => (
  <div key={text} className="whitespace-nowrap font-['Press_Start_2P'] text-sm" style={{ animation: `marquee ${duration / 1000}s linear` }}>
    {text}
  </div>
));

function App() {
  const [soul, setSoul] = useState(100.0); // Will be updated from backend
  const [news, setNews] = useState<string[]>([]);
  const [currentNewsIndex, setCurrentNewsIndex] = useState(0);
  const [stats, setStats] = useState({ total_time: 0, keys_total: 0, clicks_total: 0, mouse_total: 0 });
  const boostEnergy = async () => {
    await invoke("boost_energy");
  };

  const annoyEnergy = async () => {
    await invoke("annoy_energy");
  };

  useEffect(() => {
    verifyPermissions();

    // Set news once
    if (news.length === 0) {
      const allNews = [
        "Market crashes as developer productivity plummets",
        "Stocks soar on employee burnout rumors",
        "New study: Coding without breaks leads to 100% HP loss",
        "Tech layoffs spike after productivity tracking scandal",
        "AI replaces developers, but forgets to monitor its own HP",
        "Remote work boom: Home office casualties rise 200%",
        "Startup fails spectacularly after founder works 72 hours straight",
        "Venture capital flees companies with 'no work-life balance' policies",
        "Programmer union demands 'mandatory naptime' clause in contracts",
        "Silicon Valley therapists report 300% increase in 'code fatigue' cases",
        "Big Tech announces 'productivity guilt' as new revenue stream",
        "Tech tools company sued for 'encouraging overwork'",
        "New app promises 'work less, achieve more' – downloads hit zero",
        "Corporate wellness programs now include 'HP recovery sessions'",
        "Stock market indexes now track developer happiness levels",
        "Breaking: Company culture shifts from 'hustle' to 'human'",
        "Tech conference keynote: 'Why your HP bar should never hit zero'",
        "Startup founder retires after realizing 'work-life balance' exists",
        "AI predicts: Human developers will be obsolete by 2025... or sooner",
        "Corporate boardrooms now equipped with 'empathy chips'",
        "New law: Companies must provide 'digital detox' days",
        "Tech giants race to develop 'anti-burnout' software",
        "Employee handbook updated: 'HP > KPIs'",
        "Viral tweet: 'My HP bar is lower than my coffee levels'",
        "Startup culture evolves: 'Sleep is the new productivity hack'",
        "Big Tech dividend: 'Mental health days' now taxable",
        "Developer confession: 'I thought red HP bar was just aesthetic'",
        "Corporate retreat theme: 'Recharging our internal batteries'",
        "New metric: Employee satisfaction measured in HP points",
        "Tech unicorn valuation drops after 'overwork scandal'",
        "Breaking: Company adopts '4-day workweek'... effective immediately",
        "AI assistant reminds: 'Your HP is not infinite'",
        "Corporate slogan change: 'Work smarter, not harder... or longer'",
        "Developer memes peak: 'My HP bar is my mood ring'",
        "Startup pitch: 'We help you maintain your HP while coding'",
        "Tech conference panel: 'The future of work-life balance'",
        "Corporate email: 'Mandatory fun breaks now enforced'",
        "Venture capitalist insight: 'Invest in people, not just code'",
        "Developer diary: 'Day 1: HP 100%. Day 365: HP 0%'",
        "New app category: 'HP management tools'",
        "Corporate policy: 'No meetings during HP recovery time'",
        "Tech influencer: 'My HP bar is my productivity barometer'",
        "Startup motto: 'Code fast, live slow'",
        "Corporate wellness: 'Yoga for your HP bar'",
        "Developer confession: 'I coded through my red HP warning'",
        "Big Tech announcement: 'HP tracking banned in workplaces'",
        "New study: 'Happy developers ship better code'",
        "Corporate retreat activity: 'HP bar meditation'",
        "Tech startup: 'We make HP bars for a living'",
        "Developer tip: 'Take breaks before your HP hits zero'",
        "Corporate diversity initiative: 'Inclusive HP management'",
        "Tech conference: 'HP bars: The next big UI trend'",
        "Startup culture: 'Our HP bars are always green'",
        "Corporate memo: 'HP > overtime pay'",
        "Developer wisdom: 'A green HP bar is worth a thousand commits'",
        "Tech giant: 'Our engineers' HP bars are our biggest asset'",
        "New app: 'HP bar simulator for managers'",
        "Corporate policy: 'HP checks during performance reviews'",
        "Developer burnout: 'When your HP bar becomes your enemy'",
        "Tech startup pitch: 'Revolutionizing HP management'",
        "Corporate email signature: 'Sent with full HP'",
        "Developer meme: 'My HP bar after debugging for 8 hours'",
        "Tech conference talk: 'The psychology of HP bars'",
        "Startup perk: 'Unlimited HP recovery days'",
        "Corporate wellness program: 'HP bar yoga sessions'",
        "Developer confession: 'I ignored my red HP bar'",
        "Big Tech initiative: 'HP bar transparency for all employees'",
        "New metric: 'Team HP average'",
        "Tech unicorn: 'Our valuation is tied to employee HP'",
        "Corporate retreat: 'HP bar team-building exercises'",
        "Developer advice: 'Monitor your HP, not just your code'"
      ];
      const shuffled = [...allNews].sort(() => Math.random() - 0.5);
      setNews(shuffled);
    }

    const unlisten = listen<TickerData>("ticker-update", (event) => {
      setSoul(event.payload.soul);
      setStats({
        total_time: event.payload.total_time,
        keys_total: event.payload.keys_total,
        clicks_total: event.payload.clicks_total,
        mouse_total: event.payload.mouse_total,
      });
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [news.length]);

  useEffect(() => {
    if (news.length > 0) {
      const currentDuration = Math.max(15000, (news[currentNewsIndex]?.length || 0) * 300);
      const interval = setInterval(() => {
        setCurrentNewsIndex((prev) => (prev + 1) % news.length);
      }, currentDuration);
      return () => clearInterval(interval);
    }
  }, [news, currentNewsIndex]);

  const verifyPermissions = async () => {
    // 1. アクセシビリティ権限 (マウス/一般キー用)
    const isAccessibility = await checkAccessibilityPermission();

    // 2. 入力監視権限 (A-Zキー用) - ここが重要
    const isInputMonitoring = await checkInputMonitoringPermission();

    console.log(`Accessibility: ${isAccessibility}, InputMonitoring: ${isInputMonitoring}`);

    if (!isAccessibility || !isInputMonitoring) {
      let message = 'Boobs Ticker needs the following permissions to track your productivity:\n\n';
      if (!isAccessibility) message += '- Accessibility (for mouse and special keys)\n';
      if (!isInputMonitoring) message += '- Input Monitoring (for alphabet keys A-Z)\n\n';
      message += 'Please enable them in System Settings > Privacy & Security.';

      const confirmed = await ask(message, {
        title: 'Permissions Required',
        kind: 'warning',
        okLabel: 'Open Settings',
        cancelLabel: 'Later'
      });

      if (confirmed) {
        // 不足している権限に応じてページを開く
        if (!isAccessibility) {
          await open('x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility');
        } else if (!isInputMonitoring) {
          // 入力監視のページを試す (安定していない場合がある)
          await open('x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent');
        } else {
          // 両方不足の場合、アクセシビリティページを開く
          await open('x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility');
        }
      }
    }
  };



  const duration = news.length > 0 ? Math.max(15000, (news[currentNewsIndex]?.length || 0) * 300) : 15000;

  return (
    <div className="h-36 bg-gray-100 text-black p-2">
        <div className="flex items-center justify-between h-8">
           <div className={`text-xs font-bold font-['Press_Start_2P'] ${soul > 70 ? "text-green-500" : soul > 50 ? "text-orange-500" : soul > 20 ? "text-yellow-500" : soul > 0 ? "text-red-500" : "text-red-700"}`}>
             HP: {soul.toFixed(0)}%
           </div>
           <div className="flex-1 mx-4 overflow-hidden">
             <Ticker text={news[currentNewsIndex] || ""} duration={duration} />
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

      <div className="text-right text-[10px] text-gray-500 mt-1">
        Time: {stats.total_time}m | Keys: {stats.keys_total} | Clicks: {stats.clicks_total} | Mouse: {Math.round(stats.mouse_total / 96)} in
      </div>

      <div className="text-center text-xs text-gray-500 mt-1">
        <img src="/everyday.png" className="w-4 h-4 rounded-full inline mr-1" />
        we are <a href="#" onClick={() => openUrl("https://booby.dev/about")}><img src="/boobs.png" className="h-5 w-auto inline" /></a>
      </div>
    </div>
  );
}

export default App;
