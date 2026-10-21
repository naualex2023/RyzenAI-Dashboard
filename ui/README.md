# UI — RyzenAI Dashboard frontend

Vite + React + TypeScript, без сторонних UI-библиотек (спарклайны — свои SVG).

## Сборка

```sh
cd ui
npm install       # один раз
npm run build     # tsc --noEmit && vite build → ui/dist/
```

`server/build.rs` при компиляции сервера сканирует `../ui/dist` и **встраивает**
все ассеты в бинарник (`crate::ui_assets::ASSETS`). Поэтому:
- на целевой машине достаточно `cargo build` (если `ui/dist` уже в репо);
- `ui/dist` коммитится (исключение в `.gitignore`) — node на целевой машине не нужен;
- если нужно обновить UI — `npm run build`, затем пересобрать сервер.

Для hot-dev: `npm run dev` (проксирует `/api/*` на `http://localhost:3010`).

## Данные

| endpoint | назначение |
|---|---|
| `GET /api/snapshot` | один снимок `Snapshot` (первый рендер) |
| `GET /api/events` | SSE, каждое событие `data: <Snapshot JSON>` (живой поток) |
| `GET /api/history?sec=N` | `{sec, interval_ms, timestamps, series}` для графиков |
| `GET /api/sysdump` | `{ "<procfs path>": "<content>" }` — диагностика |

Auth: если сервер запущен с `--token T`, все вызовы требуют `?token=T`
(UI берёт токен из URL `?token=`, сохраняет в localStorage, при 401 показывает форму ввода).

## Типы

`src/types.ts` — зеркально повторяет serde-структуры из
`server/src/collect/mod.rs`. При изменении полей в Rust менять их здесь же.

## Ключи серий history

Сервер сериализует снимок и разворачивает его в плоские пути (`history::walk`):
- массивы индексируются полем `id` (temps → `temps.hwmon3.temp1.temp_c`),
  иначе `name` (disks → `disks.nvme0n1.*`, network → `network.wlp2s0.rx_kbps`),
  иначе числовым индексом (per_core → `cpu.per_core.0.usage_pct`);
- `null` (Rust `Option::None`) в серию не попадает.

Группы в UI-панели History: `cpu`, `memory`, `temps`, `disks`, `network`, `gpu`, `npu`.

## Секции

- **Header** — hostname/distro/kernel/CPU/uptime/loadavg + индикатор SSE.
- **CPU** — usage%, temp, PSI(60s), бар и пер-ядерная сетка (usage + freq).
- **Memory** — used/total, available/cached/buffers/swap, PSI.
- **Temps** — сетка сенсоров, отсортированных по роли (cpu→gpu→ssd→vrm→soc→chipset→ambient→misc).
- **Accelerators** — карточки GPU (busy/vram/power/temps) и NPU; не рисуются, если `gpu`/`npu` = null.
- **Disks** — на диск: kind/rotational, температурные чипы, строки ФС с барами used_pct.
- **Network** — интерфейсы: state, link speed, rx/tx (KB/s + пакеты/с).
- **Processes** — top по CPU и по RSS.
- **History** — выбор диапазона (60/300/900/3600 с), группы серий, чекбоксы, спарклайны.
- **Sysdump** — сырые procfs/sysfs файлы в `<details>`.

## Ограничения на живой машине

- Отсутствующие источники (нет drm/gpu, нет NPU, нет NVMe-температуры) просто не рисуются —
  секции условные по данным снимка, не по железу.
- SSE автоматически переподключается (EventSource); токен сохраняется в URL запросов.
