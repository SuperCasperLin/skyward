//! The release notes shown under About phone. Kept here as Rust string
//! literals so the font subset builder collects every glyph they use.
//! The same remarks live at length in the repository's CHANGELOG.md; this
//! is the short, on-device version, newest first.

use crate::lang::Text;

/// One version's notes: its heading and its body. A blank line in the body
/// starts a new paragraph; a single newline is one line of the paragraph.
pub struct Entry {
    pub heading: Text,
    pub body: Text,
}

pub const ENTRIES: &[Entry] = &[
    Entry {
        heading: ["SW-v0.3.1-nuwa-v0.3.2 · 2026-11-22", "SW-v0.3.1-nuwa-v0.3.2 · 2026-11-22", "SW-v0.3.1-nuwa-v0.3.2 · 2026-11-22"],
        body: [
            "在 v0.3.1 真机实测基础上做的一批手感与细节优化，上游基线不变。\n\n\
· 解密页数字键与所有滑动确认（重启、安装、刷入、擦除等）加入与锁屏上滑一致的震动反馈，滑动被拒绝时保持静默。\n\
· 点按位置的压按光圈触发更快（90 毫秒缩短到 60 毫秒），快速滑动列表仍只滚动、不闪按压块。\n\
· 新增「更新日志」入口，可在「关于本机」按版本查看各版内容；开源许可页文案更新到本版。\n\
· 修复参数页处理器名称里 ® 符号显示为小方框的问题。",
            "在 v0.3.1 真機實測基礎上做的一批手感與細節優化，上游基線不變。\n\n\
· 解密頁數字鍵與所有滑動確認（重啟、安裝、刷入、擦除等）加入與鎖屏上滑一致的震動回饋，滑動被拒絕時保持靜默。\n\
· 點按位置的壓按光圈觸發更快（90 毫秒縮短到 60 毫秒），快速滑動列表仍只滾動、不閃按壓塊。\n\
· 新增「更新日誌」入口，可在「關於手機」按版本查看各版內容；開源許可頁文案更新到本版。\n\
· 修復參數頁處理器名稱裡 ® 符號顯示為小方框的問題。",
            "A batch of feel and polish changes on top of the device-tested v0.3.1;\n\
the upstream baseline is unchanged.\n\n\
· The decrypt keypad and every swipe confirm (reboot, install, flash, wipe)\n  now share the lock screen's slide-to-unlock haptic; refused swipes stay quiet.\n\
· The touch press highlight appears sooner (90 ms down to 60 ms), while a\n  quick list scroll still just scrolls with no press flash.\n\
· A new Changelog entry under About phone lists each version; the licence\n  notice was brought up to date.\n\
· Fixed the ® symbol in the processor name showing as a missing-glyph box.",
        ],
    },
    Entry {
        heading: ["SW-v0.3.1-nuwa-v0.3.1 · 2026-11-20", "SW-v0.3.1-nuwa-v0.3.1 · 2026-11-20", "SW-v0.3.1-nuwa-v0.3.1 · 2026-11-20"],
        body: [
            "基线升到上游 Skyward v0.3.1，移植版本号随之同步，在线更新的发行号也对齐。\n\
此前各版基于上游 v0.2.0。\n\n\
· 上游这一版的界面代码做了拆分重构，并新增了若干底层组件；本移植的定制功能逐项核对后保留。\n\
· 在线更新仍指向本项目自己的发行仓库，不会跳回官方。\n\
· 开机锁与密码输入线程适配了新的主循环结构。\n\
· 本版为真机实装验证版：锁屏密码解密、安全密钥链、设置保留均已实机通过。",
            "基線升到上游 Skyward v0.3.1，移植版本號隨之同步，線上更新的發行號也對齊。\n\
此前各版基於上游 v0.2.0。\n\n\
· 上游這一版的介面程式碼做了拆分重構，並新增若干底層元件；本移植的定製功能逐項核對後保留。\n\
· 線上更新仍指向本專案自己的發行倉庫，不會跳回官方。\n\
· 開機鎖與密碼輸入執行緒適配了新的主迴圈結構。\n\
· 本版為真機實裝驗證版：鎖屏密碼解密、安全金鑰鏈、設定保留均已實機通過。",
            "Base moved up to upstream Skyward v0.3.1; the port's version and the\n\
update release number moved with it. Earlier builds used upstream v0.2.0.\n\n\
· Upstream reorganised its UI code and added low-level components; this\n  port's custom features were checked one by one and kept.\n\
· Online updates still point at this port's own repository, never upstream.\n\
· Boot lock and the passcode worker were adapted to the new main loop.\n\
· This build is verified on the device: passcode decryption, the secure\n  key chain and preserved settings all passed on the phone.",
        ],
    },
    Entry {
        heading: ["SW-v0.2.0-nuwa-v0.2.4 · 2026-11-15", "SW-v0.2.0-nuwa-v0.2.4 · 2026-11-15", "SW-v0.2.0-nuwa-v0.2.4 · 2026-11-15"],
        body: [
            "在线更新指向本项目自己的 GitHub 发行仓库；发行号与版本号统一为移植版本尾段。\n\
仓库留空时不联网、直接判定为已是最新，绝不回退到官方包。\n\n\
· 修复：在 Skyward 里刷 recovery 卡刷包会被还原成旧版（假成功）。现在安装器留下的是 Skyward 的槽一律保留。\n\
· 卡刷包改为通刷，不探测宿主、不读开关；第三方 Recovery 与 Skyward 自身都能直接刷。\n\
· 对外署名统一为 Casper Lin。",
            "線上更新指向本專案自己的 GitHub 發行倉庫；發行號與版本號統一為移植版本尾段。\n\
倉庫留空時不連網、直接判定為已是最新，絕不回退到官方包。\n\n\
· 修復：在 Skyward 裡刷 recovery 卡刷包會被還原成舊版（假成功）。現在安裝器留下的是 Skyward 的槽一律保留。\n\
· 卡刷包改為通刷，不探測宿主、不讀開關；第三方 Recovery 與 Skyward 自身都能直接刷。\n\
· 對外署名統一為 Casper Lin。",
            "Online updates now point at this port's own GitHub releases; the release\n\
number matches the port's version. With no repository set, the check stays\noffline and reports up to date, never falling back to the official package.\n\n\
· Fixed: flashing a recovery zip inside Skyward could be restored to the old\n  build. The installer now keeps any slot that still holds Skyward.\n\
· The flashable zip is now universal: no host detection, no switches read.\n\
· Public credit unified as Casper Lin.",
        ],
    },
    Entry {
        heading: ["SW-v0.2.0-nuwa-v0.2.4 · 2026-11-11", "SW-v0.2.0-nuwa-v0.2.4 · 2026-11-11", "SW-v0.2.0-nuwa-v0.2.4 · 2026-11-11"],
        body: [
            "锁屏磨砂玻璃跟着会动的页面每帧实时变化，约 62 fps。\n\n\
· 60 fps 实时动态模糊：底下页面在动时（如「关于本机」的流动光晕）每一帧重新取一次磨砂。\n\
· 静止页面仍用静态缓存，观感与耗电不变。\n\
· 修复：首次进入 Recovery 会跳过锁屏。\n\
· 修复：按电源键会直接跳到息屏显示，看不到锁屏；现在第一次按是上锁并保持亮屏。",
            "鎖屏磨砂玻璃跟著會動的頁面每幀即時變化，約 62 fps。\n\n\
· 60 fps 即時動態模糊：底下頁面在動時（如「關於手機」的流動光暈）每一幀重新取一次磨砂。\n\
· 靜止頁面仍用靜態快取，觀感與耗電不變。\n\
· 修復：首次進入 Recovery 會跳過鎖屏。\n\
· 修復：按電源鍵會直接跳到息屏顯示，看不到鎖屏；現在第一次按是上鎖並保持亮屏。",
            "The lock screen's frosted glass now follows moving pages every frame,\n\
about 62 fps.\n\n\
· Live 60 fps blur: when the page below moves (the flowing glow on About\n  phone), the glass is refreshed each frame.\n\
· Still pages keep the cached glass, with no change to looks or power.\n\
· Fixed: the first boot into Recovery could skip the lock screen.\n\
· Fixed: the power key jumped straight to the always-on display; the first\n  press now locks and keeps the panel on.",
        ],
    },
    Entry {
        heading: ["SW-v0.2.0-nuwa-v0.2.3 · 2026-11-11", "SW-v0.2.0-nuwa-v0.2.3 · 2026-11-11", "SW-v0.2.0-nuwa-v0.2.3 · 2026-11-11"],
        body: [
            "锁屏磨砂玻璃不再定格：底下页面在动时，每 250 ms 重取一次，其余帧复用缓存，静止页面一次都不重算。\n\n\
· 字色只在上锁那一帧判定一次，避免颜色漂过分界时黑白翻转。\n\
· 充电动画铺满时不重取背景。",
            "鎖屏磨砂玻璃不再定格：底下頁面在動時，每 250 ms 重取一次，其餘幀複用快取，靜止頁面一次都不重算。\n\n\
· 字色只在上鎖那一幀判定一次，避免顏色漂過分界時黑白翻轉。\n\
· 充動畫鋪滿時不重取背景。",
            "The lock screen's frosted glass no longer freezes: when the page below\nmoves, it refreshes every 250 ms; other frames reuse the cache, and still\npages never recompute.\n\n\
· Text colour is decided once on locking, so it cannot flip black/white as\n  the brightness drifts across a threshold.\n\
· The background is not refreshed while the charging animation fills the\n  screen.",
        ],
    },
    Entry {
        heading: ["SW-v0.2.0-nuwa-v0.2.2 · 2026-11-10", "SW-v0.2.0-nuwa-v0.2.2 · 2026-11-10", "SW-v0.2.0-nuwa-v0.2.2 · 2026-11-10"],
        body: [
            "给面板加锁屏：熄灭即上锁，先上滑；设了密码再输密码，没设密码上滑就直接进页面。\n\n\
· 背景是透出来的页面，做磨砂处理后像隔着玻璃看过去；字色跟着玻璃明暗自动选。\n\
· 可设解锁密码（至少 4 位，最多 8 位），用 scrypt 拉伸后保存，校验用常量时间比较。\n\
· 上滑解锁、键盘任意按键、边缘返回都会震动反馈。\n\
· 锁屏电量用蓝 / 绿 / 红三色表示剩余 / 充电 / 低电，充电时带黄色闪电。\n\
· 首页也认侧边滑动返回。\n\
· 隔着包误触只会亮屏停在锁屏，不会点到危险操作。",
            "為面板加密鎖：熄滅即上鎖，先上滑；設了密碼再輸密碼，沒設密碼上滑就直接進頁面。\n\n\
· 背景是透出來的頁面，做磨砂處理後像隔著玻璃看過去；字色跟著玻璃明暗自動選。\n\
· 可設解鎖密碼（至少 4 位，最多 8 位），用 scrypt 拉伸後儲存，校驗用常量時間比較。\n\
· 上滑解鎖、鍵盤任意按鍵、邊緣返回都會震動回饋。\n\
· 鎖屏電量用藍 / 綠 / 紅三色表示剩餘 / 充電 / 低電，充電時帶黃色閃電。\n\
· 首頁也認側邊滑動返回。\n\
· 隔著包誤觸只會亮屏停在鎖屏，不會點到危險操作。",
            "Added a lock screen: the panel locks as it goes off; swipe first, then\n\
enter a code if one is set, or swipe straight through if not.\n\n\
· The background is the page beneath, frosted like glass; text colour follows\n  the glass brightness.\n\
· An unlock code (4-8 digits) is stored with scrypt and checked in constant\n  time.\n\
· Swipe to unlock, every keypress and edge-back all give haptic feedback.\n\
· Battery shows blue / green / red for remaining / charging / low, with a\n  yellow bolt while charging.\n\
· The home page also accepts the edge swipe back.\n\
· A bump in a pocket only wakes the panel at the lock screen, never taps a\n  dangerous action.",
        ],
    },
    Entry {
        heading: ["SW-v0.2.0-nuwa-v0.2.0 · 2026-11-10", "SW-v0.2.0-nuwa-v0.2.0 · 2026-11-10", "SW-v0.2.0-nuwa-v0.2.0 · 2026-11-10"],
        body: [
            "在「能解密」的基础上补齐可用性。\n\n\
· 息屏显示（AOD）：屏幕熄灭后以约 2% 背光显示时间、日期、电量，每分钟换位防烧屏；轻触唤醒。\n\
· 大写锁定：屏幕键盘 Shift 双击锁定大写，三处键盘行为一致。\n\
· 版本号拆成上游基线与本移植版本两部分。",
            "在「能解密」的基礎上補齊可用性。\n\n\
· 息屏顯示（AOD）：螢幕熄滅後以約 2% 背光顯示時間、日期、電量，每分鐘換位防燒屏；輕觸喚醒。\n\
· 大寫鎖定：螢幕鍵盤 Shift 雙擊鎖定大寫，三處鍵盤行為一致。\n\
· 版本號拆成上游基線與本移植版本兩部分。",
            "Built on decrypting data, this build fills in the everyday details.\n\n\
· Always-on display: after the screen goes off, time, date and battery stay\n  at about 2% brightness, shifting position each minute; tap to wake.\n\
· Caps lock: double-tap Shift on the on-screen keyboard; all three keyboards\n  behave alike.\n\
· The version number is split into the upstream base and the port's own.",
        ],
    },
    Entry {
        heading: ["SW-v0.2.0-nuwa-v0.1.0 · 2026-11-10", "SW-v0.2.0-nuwa-v0.1.0 · 2026-11-10", "SW-v0.2.0-nuwa-v0.1.0 · 2026-11-10"],
        body: [
            "第一版：移植打通，data 可以解密。\n\n\
· 补足 nuwa 所需的 TEE 服务链（qseecomd、keymint、hwservicemanager、gatekeeper、bootctl），拉起 KeyMint。\n\
· /data 以 f2fs 读写挂载，识别出用户。\n\
· Wi-Fi 连接成功，拿到 IPv4 与 IPv6。\n\
· 备份：基带与 EFS 分区，校验一致。",
            "第一版：移植打通，data 可以解密。\n\n\
· 補足 nuwa 所需的 TEE 服務鏈（qseecomd、keymint、hwservicemanager、gatekeeper、bootctl），拉起 KeyMint。\n\
· /data 以 f2fs 讀寫掛載，識別出使用者。\n\
· Wi-Fi 連線成功，取得 IPv4 與 IPv6。\n\
· 備份：基頻與 EFS 分割區，校驗一致。",
            "First build: the port works and data can be decrypted.\n\n\
· Added the TEE service chain nuwa needs (qseecomd, keymint,\n  hwservicemanager, gatekeeper, bootctl) to bring up KeyMint.\n\
· /data mounts read-write as f2fs and its users are recognised.\n\
· Wi-Fi connects and gets IPv4 and IPv6 addresses.\n\
· Backup covers the baseband and EFS partitions, verified against the\n  manifest.",
        ],
    },
];
