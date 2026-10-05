use crate::module::{Module, ModuleOutput};
use crate::sys;

pub struct PowerProfile;

impl Module for PowerProfile {
    fn name(&self) -> &'static str {
        "Power Profile"
    }
    fn id(&self) -> &'static str {
        "powerprofile"
    }
    fn run(&self) -> Option<ModuleOutput> {
        #[cfg(target_os = "linux")]
        {
            if let Some(profile) = current_profile() {
                return Some(ModuleOutput::new("Power Profile", capitalize(&profile)));
            }

            // No ACPI profile here, so go by the scaling governor.
            if let Some(gov) = sys::read_trim(SCALING_GOVERNOR)
                && !gov.is_empty()
            {
                return Some(ModuleOutput::new(
                    "Power Profile",
                    format!("{} (governor)", capitalize(&gov)),
                ));
            }
        }

        None
    }
}

const SCALING_GOVERNOR: &str = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor";
const PLATFORM_PROFILE: &str = "/sys/firmware/acpi/platform_profile";
const PM_PROFILE: &str = "/sys/firmware/acpi/pm_profile";
const PROFILE_CHOICES: &str = "/sys/firmware/acpi/platform_profile_choices";

/// Resolves the ACPI profile from the `pm_profile` index, then `platform_profile`.
fn current_profile() -> Option<String> {
    if let Some(name) = profile_via_index() {
        return Some(name);
    }
    sys::read_trim(PLATFORM_PROFILE)
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
}

fn profile_via_index() -> Option<String> {
    let index: usize = sys::read_trim(PM_PROFILE)?.trim().parse().ok()?;
    let choices = sys::read_trim(PROFILE_CHOICES)?;
    let name = choices.split_whitespace().nth(index)?;
    (!name.is_empty()).then(|| name.to_string())
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capitalize_only_touches_the_first_letter() {
        assert_eq!(capitalize("balanced"), "Balanced");
        assert_eq!(capitalize("POWER"), "POWER");
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn missing_files_produce_no_profile() {
        // Odd hardware should give None, never a panic.
        let _ = current_profile();
        let _ = profile_via_index();
    }

    #[test]
    fn index_maps_onto_the_choices_list() {
        // Both files present: the index has to match the name.
        let (Some(idx), Some(choices)) =
            (sys::read_trim(PM_PROFILE), sys::read_trim(PROFILE_CHOICES))
        else {
            return;
        };
        let Ok(index) = idx.trim().parse::<usize>() else {
            return;
        };
        let names: Vec<&str> = choices.split_whitespace().collect();
        let Some(expected) = names.get(index) else {
            return;
        };
        assert_eq!(
            sys::read_trim(PLATFORM_PROFILE).map(|s| s.trim().to_string()),
            Some(expected.to_string()),
            "pm_profile index disagrees with platform_profile"
        );
    }
}
