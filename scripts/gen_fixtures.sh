#!/usr/bin/env bash
# Deterministic synthetic /proc + /sys + /etc snapshot for server unit tests.
# Values mirror the assertions in src/**/tests (16 GB RAM, 8 CPUs, nvme0n1,
# wlp2s0 + tailscale0, k10temp/nvme/amdgpu/acpitz hwmon 0-3, card1 AMD, …).
set -euo pipefail

F="$(dirname "$0")/../server/fixtures/sys"
rm -rf "$F"
mkdir -p \
  "$F/proc/42" "$F/proc/43" "$F/proc/net" "$F/proc/pressure" "$F/proc/sys/kernel" \
  "$F/etc" \
  "$F/sys/class/hwmon/hwmon0" "$F/sys/class/hwmon/hwmon1" \
  "$F/sys/class/hwmon/hwmon2" "$F/sys/class/hwmon/hwmon3" \
  "$F/sys/class/thermal/thermal_zone0" "$F/sys/class/thermal/thermal_zone1" \
  "$F/sys/class/drm/card1/device" \
  "$F/sys/class/block/nvme0n1/queue" "$F/sys/class/block/nvme0n1p1" "$F/sys/class/block/loop0" \
  "$F/sys/class/net/wlp2s0" "$F/sys/class/net/tailscale0"

# ----- /proc -----
printf 'cpu 1000 500 900 20000 100 50 60 0 0 0\n' > "$F/proc/stat"
for c in 0 1 2 3 4 5 6 7; do
  printf 'cpu%s %s %s %s %s %s %s 0 0 0\n' "$c" $((125 - c)) $((60 - c % 8)) $((110 - c % 11)) 2500 12 6 7 >> "$F/proc/stat"
done
printf 'btime 1755000000\n' >> "$F/proc/stat"

cat > "$F/proc/meminfo" << 'EOF'
MemTotal:      16000000 kB
MemFree:       8000000 kB
MemAvailable:  8000000 kB
Buffers:       100000 kB
Cached:        2500000 kB
SReclaimable:  300000 kB
SwapTotal:     8000000 kB
SwapFree:      8000000 kB
EOF

printf '12345.67 45000.89\n' > "$F/proc/uptime"
printf '0.50 1.20 0.80 8/1234 42\n' > "$F/proc/loadavg"
printf 'Linux version 6.11.0-1008-amd64 (alex@build) #42-Ubuntu SMP Thu Jan 1 00:00:00 UTC 2026\n' > "$F/proc/version"
printf 'processor : 0\nmodel name : AMD Ryzen AI Max+ 395\n' > "$F/proc/cpuinfo"

printf '252 0 nvme0n1 100 204800 10 500 100000 5 0 3000 1500\n' > "$F/proc/diskstats"
printf '252 1 nvme0n1p1 50 102400 5 200 50000 2 0 1200 700\n' >> "$F/proc/diskstats"
printf '7 0 loop0 0 0 0 0 0 0 0 0 0\n' >> "$F/proc/diskstats"

cat > "$F/proc/net/dev" << 'EOF'
Inter-|   Enhanced
face   rx bytes rx packets 0 0 0 0 0 0 tx bytes tx packets 0 0 0 0 0 0
lo: 12345 78 0 0 0 0 0 0 12345 78 0 0 0 0 0 0
wlp2s0: 900000 700 0 0 0 0 0 0 150000 600 0 0 0 0 0 0
tailscale0: 100000 400 0 0 0 0 0 0 90000 350 0 0 0 0 0 0
docker0: 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
EOF

printf 'some avg10=0.50 avg60=0.40 avg300=0.30\nfull avg10=0.00 avg60=0.00 avg300=0.00\n' > "$F/proc/pressure/cpu"
printf 'some avg10=0.20 avg60=0.10 avg300=0.05\nfull avg10=0.00 avg60=0.00 avg300=0.00\n' > "$F/proc/pressure/memory"
printf 'some avg10=0.05 avg60=0.04 avg300=0.02\nfull avg10=0.00 avg60=0.00 avg300=0.00\n' > "$F/proc/pressure/io"

# /proc/PID/stat: 24 fields; utime @13, stime @14, rss @23 (pages)
printf '42 (sleep) S 1 42 42 0 -1 41 0 0 0 0 0 100 5 0 0 0 0 0 0 0 0 12\n' > "$F/proc/42/stat"
printf '43 (bash) R 0 43 43 4 43 43 0 0 0 0 0 50 5 0 0 0 0 0 0 0 0 8\n' > "$F/proc/43/stat"

printf 'devbox\n' > "$F/proc/sys/kernel/hostname"

printf '/dev/nvme0n1p1 / ext4 rw,relatime 0 0\n' > "$F/proc/mounts"
printf '/dev/nvme0n1p2 /home ext4 rw 0 0\n' >> "$F/proc/mounts"
printf 'tmpfs /tmp tmpfs rw 0 0\n' >> "$F/proc/mounts"

# ----- /etc -----
cat > "$F/etc/os-release" << 'EOF'
ID=ubuntu
PRETTY_NAME="Ubuntu 24.04 LTS (fixture)"
EOF
printf 'devbox\n' > "$F/etc/hostname"

# ----- /sys/class/hwmon (contiguous hwmon0-3) -----
printf 'k10temp\n'  > "$F/sys/class/hwmon/hwmon0/name"
printf '45000\n'    > "$F/sys/class/hwmon/hwmon0/temp1_input"
printf 'Tctl\n'     > "$F/sys/class/hwmon/hwmon0/temp1_label"
printf 'nvme\n'     > "$F/sys/class/hwmon/hwmon1/name"
printf '38000\n'    > "$F/sys/class/hwmon/hwmon1/temp1_input"
printf 'Composite\n' > "$F/sys/class/hwmon/hwmon1/temp1_label"
printf 'amdgpu\n'   > "$F/sys/class/hwmon/hwmon2/name"
printf '55000\n'    > "$F/sys/class/hwmon/hwmon2/temp1_input"
printf 'edge\n'     > "$F/sys/class/hwmon/hwmon2/temp1_label"
printf '5400000\n'  > "$F/sys/class/hwmon/hwmon2/power1_input"
printf 'acpitz\n'   > "$F/sys/class/hwmon/hwmon3/name"
printf '30850\n'    > "$F/sys/class/hwmon/hwmon3/temp1_input"
printf 'acpitz_0\n' > "$F/sys/class/hwmon/hwmon3/temp1_label"

# ----- /sys/class/thermal -----
printf '30000\n' > "$F/sys/class/thermal/thermal_zone0/temp"
printf 'acpitz 0\n' > "$F/sys/class/thermal/thermal_zone0/type"
printf '40000\n' > "$F/sys/class/thermal/thermal_zone1/temp"
printf 'soc_thermal\n' > "$F/sys/class/thermal/thermal_zone1/type"

# ----- /sys/class/drm (AMD card1, no perf/VRAM accounting) -----
printf '0x1002 0x13f6\n' > "$F/sys/class/drm/card1/device/vendor"

# ----- /sys/class/block -----
printf '0\n' > "$F/sys/class/block/nvme0n1/queue/rotational"
printf '1\n' > "$F/sys/class/block/nvme0n1p1/partition"

# ----- /sys/class/net -----
printf '86600\n' > "$F/sys/class/net/wlp2s0/speed"
printf 'up\n' > "$F/sys/class/net/wlp2s0/operstate"
printf '%s\n' '-1' > "$F/sys/class/net/tailscale0/speed"
printf 'unknown\n' > "$F/sys/class/net/tailscale0/operstate"

echo "fixtures generated under $F"