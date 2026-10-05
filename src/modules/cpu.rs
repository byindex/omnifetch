use crate::config;
use crate::module::{Module, ModuleOutput};
use crate::sys;
use crate::template;

fn get_cpuinfo() -> Option<&'static str> {
    static CPUINFO: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    CPUINFO
        .get_or_init(|| std::fs::read_to_string("/proc/cpuinfo").ok())
        .as_deref()
}

pub struct Cpu;

impl Module for Cpu {
    fn name(&self) -> &'static str {
        "CPU"
    }
    fn id(&self) -> &'static str {
        "cpu"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let info = get_cpuinfo()?;
        let mut model: Option<String> = None;
        let mut vendor = "";
        let mut family: u32 = 0;
        let mut model_id: u32 = 0;
        let mut flags = "";

        for line in info.lines() {
            if let Some((k, v)) = line.split_once(':') {
                let (k, v) = (k.trim(), v.trim());
                match k {
                    "model name" if model.is_none() => model = Some(v.to_string()),
                    "vendor_id" if vendor.is_empty() => vendor = v,
                    "cpu family" if family == 0 => family = v.parse().unwrap_or(0),
                    "model" if model_id == 0 => model_id = v.parse().unwrap_or(0),
                    "flags" if flags.is_empty() => flags = v,
                    _ => {}
                }
                if model.is_some()
                    && !vendor.is_empty()
                    && family != 0
                    && model_id != 0
                    && !flags.is_empty()
                {
                    break;
                }
            }
        }
        let model = model.unwrap_or_else(|| "Unknown CPU".to_string());

        let logical = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(0);
        let physical = physical_cores().unwrap_or(logical);

        let cores = if physical == logical {
            format!("{logical} cores")
        } else {
            format!("{physical}c/{logical}t")
        };

        let freq = current_freq_mhz();
        let microarch = detect_microarch(vendor, family, model_id);
        let simd = detect_simd(flags);
        let simd_str = simd.join(", ");
        let peak_gflops = freq.map(|f| calculate_peak_gflops(logical, f, flags));
        let gflops_str = peak_gflops
            .map(|g| format!("{g:.1} GFLOP/s"))
            .unwrap_or_default();

        let default_tmpl = if microarch.is_some() {
            "{name} ({cores}) [{uarch}] @ {freq}"
        } else {
            "{name} ({cores}) @ {freq}"
        };

        let tmpl = config::get().format("cpu").unwrap_or(default_tmpl);

        let mut ctx = template::Context::new("cpu");
        ctx.set_str("name", model.clone());
        ctx.set_str("n", model);
        ctx.set_str("cores", cores.clone());
        ctx.set_str("c", cores);
        if let Some(f) = freq {
            ctx.set_str("freq", format!("{f} MHz"));
            ctx.set_str("f", format!("{f} MHz"));
            ctx.set_num("mhz", f as f64);
        } else {
            ctx.set_str("freq", "");
            ctx.set_str("f", "");
        }
        ctx.set_num("pcores", physical as f64);
        ctx.set_num("lcores", logical as f64);

        if let Some(uarch) = microarch {
            ctx.set_str("microarch", uarch);
            ctx.set_str("uarch", uarch);
        } else {
            ctx.set_str("microarch", "");
            ctx.set_str("uarch", "");
        }
        ctx.set_str("simd", simd_str);
        ctx.set_str("gflops", gflops_str.clone());
        ctx.set_str("peak_gflops", gflops_str);

        let formatted = template::render_template(tmpl, &ctx);
        Some(ModuleOutput::new("CPU", formatted))
    }
}

pub fn detect_microarch(vendor: &str, family: u32, model: u32) -> Option<&'static str> {
    if vendor == "GenuineIntel" && family == 6 {
        match model {
            26 | 30 | 31 | 46 => Some("Nehalem"),
            37 | 44 | 47 => Some("Westmere"),
            42 | 45 => Some("Sandy Bridge"),
            58 | 62 => Some("Ivy Bridge"),
            60 | 63 | 69 | 70 => Some("Haswell"),
            61 | 71 | 79 | 86 => Some("Broadwell"),
            78 | 94 | 142 | 158 | 165 | 166 => Some("Skylake"),
            106 | 125 | 126 => Some("Ice Lake"),
            140 | 141 => Some("Tiger Lake"),
            151 | 154 => Some("Alder Lake"),
            183 | 186 => Some("Raptor Lake"),
            170 => Some("Meteor Lake"),
            197 => Some("Arrow Lake"),
            _ => None,
        }
    } else if vendor == "AuthenticAMD" {
        match family {
            23 => match model {
                1 | 17 => Some("Zen"),
                8 | 24 => Some("Zen+"),
                49 | 113 | 144 => Some("Zen 2"),
                _ => Some("Zen"),
            },
            25 => match model {
                1 | 33 | 80 => Some("Zen 3"),
                17 | 96 | 116 => Some("Zen 4"),
                _ => Some("Zen 3/4"),
            },
            26 => Some("Zen 5"),
            _ => None,
        }
    } else {
        None
    }
}

pub fn detect_simd(flags: &str) -> Vec<&'static str> {
    let mut simd = Vec::new();
    if flags.contains("avx512f") {
        simd.push("AVX-512");
    } else if flags.contains("avx2") {
        simd.push("AVX2");
    } else if flags.contains("avx") {
        simd.push("AVX");
    } else if flags.contains("sse4_2") {
        simd.push("SSE4.2");
    }
    if flags.contains("fma") {
        simd.push("FMA3");
    }
    if flags.contains("aes") {
        simd.push("AES");
    }
    if flags.contains("sha_ni") {
        simd.push("SHA");
    }
    simd
}

pub fn calculate_peak_gflops(logical_cores: usize, freq_mhz: u64, flags: &str) -> f64 {
    let ops_per_cycle = if flags.contains("avx512f") {
        32.0
    } else if flags.contains("fma") || flags.contains("avx2") {
        16.0
    } else if flags.contains("avx") {
        8.0
    } else {
        4.0
    };
    (logical_cores as f64) * (freq_mhz as f64 / 1000.0) * ops_per_cycle
}

fn physical_cores() -> Option<usize> {
    let info = get_cpuinfo()?;
    let mut seen = [0u64; 8];
    let mut count = 0usize;
    let mut current_pkg = 0usize;
    for line in info.lines() {
        if let Some((k, v)) = line.split_once(':') {
            let (k, v) = (k.trim(), v.trim());
            match k {
                "physical id" => current_pkg = v.parse::<usize>().unwrap_or(0).min(7),
                "core id" => {
                    let core: usize = v.parse().unwrap_or(0);
                    if core < 64 {
                        let mask = 1u64 << core;
                        if (seen[current_pkg] & mask) == 0 {
                            seen[current_pkg] |= mask;
                            count += 1;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    (count > 0).then_some(count)
}

fn current_freq_mhz() -> Option<u64> {
    if let Some(khz) = sys::read_num("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq") {
        return Some((khz / 1000.0).round() as u64);
    }
    let info = get_cpuinfo()?;
    let mhz = info
        .lines()
        .find(|l| l.starts_with("cpu MHz"))
        .and_then(|l| l.split_once(':'))?
        .1
        .trim()
        .parse::<f64>()
        .ok()?;
    Some(mhz.round() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_microarch_intel() {
        assert_eq!(detect_microarch("GenuineIntel", 6, 58), Some("Ivy Bridge"));
        assert_eq!(detect_microarch("GenuineIntel", 6, 142), Some("Skylake"));
        assert_eq!(
            detect_microarch("GenuineIntel", 6, 186),
            Some("Raptor Lake")
        );
    }

    #[test]
    fn test_detect_microarch_amd() {
        assert_eq!(detect_microarch("AuthenticAMD", 25, 33), Some("Zen 3"));
        assert_eq!(detect_microarch("AuthenticAMD", 23, 113), Some("Zen 2"));
    }

    #[test]
    fn test_detect_simd() {
        let flags = "fpu vme de sse sse2 sse4_1 sse4_2 avx aes";
        let simd = detect_simd(flags);
        assert!(simd.contains(&"AVX"));
        assert!(simd.contains(&"AES"));
    }
}
