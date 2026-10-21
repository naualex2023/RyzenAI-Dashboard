//! Curated sensor classification: hwmon/thermal names → dashboard roles.
//!
//! Discovery is generic (any hwmon or thermal_zone), classification is by
//! known chip names (`k10temp`, `amdgpu`, `nvme*`, …) and label patterns
//! (`Tctl`, `Composite`, `edge`, `junction`, VRM groups, …). Unknown sources
//! fall back to the general temperature grid so nothing is ever hidden.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    Cpu,
    Soc,
    Gpu,
    Ssd,
    Npu,
    Vrm,
    Chipset,
    Ambient,
    Misc,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Cpu => "cpu",
            Role::Soc => "soc",
            Role::Gpu => "gpu",
            Role::Ssd => "ssd",
            Role::Npu => "npu",
            Role::Vrm => "vrm",
            Role::Chipset => "chipset",
            Role::Ambient => "ambient",
            Role::Misc => "misc",
        }
    }

    /// Sort order for the temperature grid (SSD first by popular request).
    /// Note: the collector currently keeps its own string-keyed `role_rank`
    /// table (see `collect/mod.rs`); `rank` is reserved for typed sorting.
    #[allow(dead_code)]
    pub fn rank(&self) -> u8 {
        match self {
            Role::Ssd => 0,
            Role::Cpu => 1,
            Role::Gpu => 2,
            Role::Npu => 3,
            Role::Soc => 4,
            Role::Vrm => 5,
            Role::Chipset => 6,
            Role::Ambient => 7,
            Role::Misc => 8,
        }
    }
}

/// Classify by source name (chip) and, optionally, sensor label.
/// `name` is the hwmon `name` file / thermal zone `type` file content.
pub fn classify(name: &str, label: &str) -> Role {
    let n = name.to_lowercase();
    let l = label.to_lowercase();

    // chip-level rules first
    if n.contains("k10temp")
        || n.contains("coretemp")
        || n.contains("cpu_thermal")
        || n.starts_with("cpu")
        || n.contains("x86_pkg_temp")
        || n.contains("pkg_temp")
    {
        return Role::Cpu;
    }
    if n.contains("amdgpu") {
        return Role::Gpu;
    }
    if n.starts_with("nvme") {
        return Role::Ssd;
    }
    if n.contains("npu") || n.contains("accel") {
        return Role::Npu;
    }
    if n.contains("pch") {
        return Role::Chipset;
    }
    if n.contains("acpitz") || n.contains("acpi") || n.contains("ambient") {
        return Role::Ambient;
    }
    if n.contains("soc") {
        return Role::Soc;
    }

    // label-level fallbacks (also useful for thermal_zone types).
    let l = l.as_str();
    // VRM families come first: labels like `VCR_CPU_VCORE` are VRM rails,
    // not generic CPU sensors — the `cpu` match must not shadow them.
    if l.contains("vrm")
        || l.contains("vcore")
        || l.contains("vcr")
        || l.contains("vpp")
        || l.contains("vr_")
    {
        return Role::Vrm;
    }
    if l.contains("tctl") || l.contains("tccd") || l.contains("cpu") {
        return Role::Cpu;
    }
    if l.contains("edge") || l.contains("junction") || l.contains("gpu") {
        return Role::Gpu;
    }
    if l.contains("composite") || l.contains("intern") || l.contains("nv") && l.contains("nvme") {
        return Role::Ssd;
    }
    if l.contains("npu") {
        return Role::Npu;
    }
    if l.contains("soc") {
        return Role::Soc;
    }
    if l.contains("pch") || l.contains("chipset") || l.contains("motherboard") {
        return Role::Chipset;
    }
    if l.contains("ambient") || l.contains("acpitz") {
        return Role::Ambient;
    }
    Role::Misc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curated_names() {
        assert_eq!(classify("k10temp", "Tctl"), Role::Cpu);
        assert_eq!(classify("amdgpu", "edge"), Role::Gpu);
        assert_eq!(classify("amdgpu", "junction"), Role::Gpu);
        assert_eq!(classify("nvme", "Composite"), Role::Ssd);
        assert_eq!(classify("nvme_composite", "Sensor 1"), Role::Ssd);
        assert_eq!(classify("acpitz_0", ""), Role::Ambient);
        assert_eq!(classify("pch_skylake", ""), Role::Chipset);
        assert_eq!(classify("npu1", "NPU"), Role::Npu);
        assert_eq!(classify("soc_thermal", ""), Role::Soc);
        assert_eq!(classify("battery", "PSU"), Role::Misc);
    }

    #[test]
    fn label_fallbacks() {
        assert_eq!(classify("", "Tccd1"), Role::Cpu);
        assert_eq!(classify("unknown", "VCR_CPU_VCORE"), Role::Vrm);
        assert_eq!(classify("thermal-fallback", "GPU edge"), Role::Gpu);
    }

    #[test]
    fn rank_prefers_ssd() {
        assert!(Role::Ssd.rank() < Role::Cpu.rank());
    }
}