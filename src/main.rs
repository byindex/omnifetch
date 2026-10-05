use omnifetch::cache::{self, StaticCache};
use omnifetch::config::{self, Config, Logo, Parsed};
use omnifetch::module::ModuleOutput;
use omnifetch::{BoxedModule, logo, modules, render, sys};

/// Result of running one module: its id, output, and duration in microseconds.
type ModuleRun = (&'static str, Option<ModuleOutput>, u128);

fn main() {
    // Writing into a closed pipe is normal here: `omnifetch --json | jq .` and
    // `omnifetch | head` both make the reader go away first. Rust turns that
    // into a panic, so SIGPIPE goes back to its default and the process just
    // stops writing.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut cfg = match config::parse(&args) {
        Parsed::Help => {
            println!("{}", config::HELP);
            return;
        }
        Parsed::Version => {
            println!("omnifetch {}", config::VERSION);
            return;
        }
        Parsed::GenConfig { path, force } => {
            if path.as_deref() == Some("-") {
                print!("{}", config::DEFAULT_CONFIG_TEMPLATE);
                return;
            }
            let target_path = path
                .map(std::path::PathBuf::from)
                .unwrap_or_else(config::default_config_write_path);

            if target_path.exists() && !force {
                use std::io::{self, Write};
                print!(
                    "Config file already exists at {}. Overwrite? (y/N): ",
                    target_path.display()
                );
                let _ = io::stdout().flush();
                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_err()
                    || !matches!(input.trim().to_ascii_lowercase().as_str(), "y" | "yes")
                {
                    println!("Aborted.");
                    return;
                }
            }

            let is_tty = unsafe {
                libc::isatty(libc::STDIN_FILENO) == 1 && libc::isatty(libc::STDOUT_FILENO) == 1
            };
            if is_tty {
                if let Err(e) = omnifetch::tui_gen_config::run_interactive(&target_path) {
                    eprintln!("omnifetch: {e}");
                    std::process::exit(1);
                }
                return;
            }

            if let Some(parent) = target_path.parent()
                && let Err(e) = std::fs::create_dir_all(parent)
            {
                eprintln!(
                    "omnifetch: failed to create directory {}: {e}",
                    parent.display()
                );
                std::process::exit(1);
            }
            if let Err(e) = std::fs::write(&target_path, config::DEFAULT_CONFIG_TEMPLATE) {
                eprintln!("omnifetch: failed to write {}: {e}", target_path.display());
                std::process::exit(1);
            }
            println!("Generated config file written to {}", target_path.display());
            return;
        }
        Parsed::ListThemes => {
            println!("Available built-in themes:\n");
            for t in omnifetch::style::THEMES {
                println!("  {t}");
            }
            return;
        }
        Parsed::ListPresets => {
            println!("Available layout presets:\n");
            let width = omnifetch::presets::PRESETS
                .iter()
                .map(|p| p.name.len())
                .max()
                .unwrap_or(0);
            for p in omnifetch::presets::PRESETS {
                println!("  {:width$}  {}", p.name, p.description, width = width);
            }
            return;
        }
        Parsed::Completion(shell) => {
            match shell.to_ascii_lowercase().as_str() {
                "bash" => print!("{}", omnifetch::completion::generate_bash()),
                "zsh" => print!("{}", omnifetch::completion::generate_zsh()),
                "fish" => print!("{}", omnifetch::completion::generate_fish()),
                other => {
                    eprintln!(
                        "omnifetch: unsupported shell '{other}'. Supported: bash, zsh, fish."
                    );
                    std::process::exit(1);
                }
            }
            return;
        }
        Parsed::Error(e) => {
            eprintln!("omnifetch: {e}\nTry `omnifetch --help`.");
            std::process::exit(2);
        }
        Parsed::Run(c) => *c,
    };

    if let Some(ref w) = cfg.warning {
        eprintln!("omnifetch: warning: {w}");
    }

    if let Some(ref th) = cfg.theme {
        if let Some(theme) = omnifetch::style::get_theme(th) {
            cfg.style = theme.style;
            cfg.bar.fill = theme.bar_fill.to_string();
            cfg.bar.empty = theme.bar_empty.to_string();
        } else {
            eprintln!("omnifetch: unknown theme '{th}'. Run `omnifetch --list-themes`.");
            std::process::exit(1);
        }
    }

    if cfg.color && !sys::supports_color() {
        cfg.color = false;
    }

    config::set_global(cfg);
    let cfg = config::get();

    if cfg.list {
        for &id in modules::ALL_MODULE_IDS {
            println!("{id}");
        }
        return;
    }

    let cache = if cfg.no_cache {
        None
    } else {
        StaticCache::get().cloned()
    };

    let selected = select(cfg);

    let run_module = |m: &BoxedModule| -> ModuleRun {
        if let Some(ref c) = cache
            && cache::is_static(m.id())
            && let Some(cached) = c.modules.get(m.id())
        {
            return (m.id(), cached.clone(), 0);
        }
        if cfg.timing {
            let t = std::time::Instant::now();
            let out = m.run();
            (m.id(), out, t.elapsed().as_micros())
        } else {
            (m.id(), m.run(), 0)
        }
    };

    let started = if cfg.timing {
        Some(std::time::Instant::now())
    } else {
        None
    };
    let mut timed_slots: Vec<Option<ModuleRun>> = vec![None; selected.len()];
    let mut to_run = Vec::new();

    for (i, m) in selected.iter().enumerate() {
        if let Some(ref c) = cache
            && cache::is_static(m.id())
            && let Some(cached) = c.modules.get(m.id())
        {
            timed_slots[i] = Some((m.id(), cached.clone(), 0));
            continue;
        }
        to_run.push((i, m));
    }

    if !to_run.is_empty() {
        let has_network = to_run
            .iter()
            .any(|(_, m)| m.id() == "publicip" || m.id() == "weather");
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(to_run.len())
            .min(16);
        // Running <= 8 modules serially avoids ~0.3-0.5 ms of clone3 and thread management overhead
        if !has_network && (cfg.fast || to_run.len() <= 8 || num_threads <= 1) {
            for (i, m) in to_run {
                timed_slots[i] = Some(run_module(m));
            }
        } else {
            let next_idx = std::sync::atomic::AtomicUsize::new(0);
            std::thread::scope(|s| {
                let to_run_ref = &to_run;
                let run_ref = &run_module;
                let mut handles = Vec::with_capacity(num_threads);
                for _ in 0..num_threads {
                    handles.push(s.spawn(|| {
                        let mut local = Vec::new();
                        loop {
                            let idx = next_idx.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            if idx >= to_run_ref.len() {
                                break;
                            }
                            let (orig_idx, m) = to_run_ref[idx];
                            local.push((orig_idx, run_ref(m)));
                        }
                        local
                    }));
                }
                for h in handles {
                    if let Ok(local) = h.join() {
                        for (idx, res) in local {
                            timed_slots[idx] = Some(res);
                        }
                    }
                }
            });
        }
    }

    let timed: Vec<ModuleRun> = timed_slots.into_iter().flatten().collect();
    if let Some(st) = started {
        let elapsed = st.elapsed();
        let mut rows: Vec<_> = timed.iter().collect();
        rows.sort_by_key(|(_, _, us)| std::cmp::Reverse(*us));
        for (id, out, us) in &rows {
            eprintln!(
                "{:>8.3} ms  {}{}",
                *us as f64 / 1000.0,
                id,
                if out.is_none() { "  (empty)" } else { "" }
            );
        }
        eprintln!("{:>8.3} ms  <total>", elapsed.as_secs_f64() * 1000.0);
    }

    let results: Vec<(&'static str, Option<ModuleOutput>)> =
        timed.into_iter().map(|(id, out, _)| (id, out)).collect();

    let mut cache_dirty = false;
    let mut cache_obj = cache.unwrap_or_else(|| {
        cache_dirty = true;
        StaticCache::new()
    });

    if cache_obj.mountpoints.is_empty() {
        cache_obj.mountpoints = omnifetch::modules::disk::detect_mountpoints();
        cache_dirty = true;
    }
    if cache_obj.has_battery.is_none() {
        let has = std::path::Path::new("/sys/class/power_supply")
            .read_dir()
            .ok()
            .map(|entries| {
                entries.flatten().any(|e| {
                    omnifetch::sys::read_trim(e.path().join("type")).as_deref() == Some("Battery")
                })
            })
            .unwrap_or(false);
        cache_obj.has_battery = Some(has);
        cache_dirty = true;
    }
    if cache_obj.cputemp_path.is_none()
        && let Some(p) = omnifetch::modules::cputemp::detect_cpu_temp_path()
    {
        cache_obj.cputemp_path = Some(p);
        cache_dirty = true;
    }
    if let Some(wc) = cache::StaticCache::get_weather_cache()
        && cache_obj.weather_cache.as_ref() != Some(&wc)
    {
        cache_obj.weather_cache = Some(wc);
        cache_dirty = true;
    }

    for (id, out) in &results {
        if !cache::is_static(id) || cache_obj.modules.contains_key(*id) {
            continue;
        }
        // Empty usually means "not attached right now". Caching that freezes the answer.
        let has_value = out
            .as_ref()
            .is_some_and(|o| o.fields.iter().any(|f| !f.value.trim().is_empty()));
        if has_value {
            cache_obj.modules.insert((*id).to_string(), out.clone());
            cache_dirty = true;
        }
    }

    let image_render = if let Logo::Image(ref path) = cfg.logo {
        omnifetch::image::load_image(path, cfg.image_cols, cfg.image_rows)
    } else {
        None
    };

    let art = if image_render.is_none() {
        pick_logo(cfg, &mut cache_obj, &mut cache_dirty)
    } else {
        None
    };

    use std::io::Write;
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();

    if cfg.json {
        let json = render::to_json(&results);
        let _ = lock.write_all(json.as_bytes());
        let _ = lock.write_all(b"\n");
        let _ = lock.flush();
        if !cfg.no_cache && cache_dirty {
            cache_obj.os_mtime = cache::detect_os_mtime();
            cache_obj.packages_mtime = cache::detect_packages_mtime();
            cache_obj.desktop_mtime = cache::detect_desktop_mtime();
            cache_obj.audio_mtime = cache::detect_audio_mtime();
            if cache_obj.modules.contains_key("devenv") {
                cache_obj.devenv_mtime = cache::detect_devenv_mtime();
            }
            cache_obj.save();
        }
        unsafe {
            libc::_exit(0);
        }
    }

    let text = render::to_text(
        &results,
        art.as_deref(),
        image_render.as_ref(),
        cfg,
        &cfg.style,
    );
    let _ = lock.write_all(text.as_bytes());
    let _ = lock.write_all(b"\n");
    let _ = lock.flush();

    if !cfg.no_cache && cache_dirty {
        cache_obj.os_mtime = cache::detect_os_mtime();
        cache_obj.packages_mtime = cache::detect_packages_mtime();
        cache_obj.desktop_mtime = cache::detect_desktop_mtime();
        cache_obj.audio_mtime = cache::detect_audio_mtime();
        if cache_obj.modules.contains_key("devenv") {
            cache_obj.devenv_mtime = cache::detect_devenv_mtime();
        }
        cache_obj.save();
    }

    unsafe {
        libc::_exit(0);
    }
}

fn select(cfg: &Config) -> Vec<BoxedModule> {
    let mut mods = if cfg.fast {
        omnifetch::fast_modules()
    } else if cfg.all {
        modules::all()
            .into_iter()
            .filter(|m| m.id() != "weather" && m.id() != "publicip")
            .collect()
    } else if let Some(ref p) = cfg.preset {
        if let Some(mods) = omnifetch::presets::get(p) {
            mods
        } else {
            eprintln!(
                "omnifetch: unknown preset '{p}'. Run `omnifetch --list-presets` to see the available layouts."
            );
            std::process::exit(1);
        }
    } else if let Some(wanted) = &cfg.modules {
        if wanted.iter().any(|w| w == "all") {
            modules::all()
                .into_iter()
                .filter(|m| m.id() != "weather" && m.id() != "publicip")
                .collect()
        } else if wanted.is_empty() {
            eprintln!(
                "omnifetch: error: -m/--modules requires at least one module name. Run `omnifetch --list-modules` to see available modules."
            );
            std::process::exit(1);
        } else {
            let mut result = Vec::new();
            for w in wanted {
                if let Some(m) = modules::create_module(w) {
                    result.push(m);
                } else {
                    let suggestion = modules::find_closest_module(w);
                    if let Some(closest) = suggestion {
                        eprintln!(
                            "omnifetch: unknown module '{w}'. Did you mean '{closest}'? Run `omnifetch --list-modules` to see available modules."
                        );
                    } else {
                        eprintln!(
                            "omnifetch: unknown module '{w}'. Run `omnifetch --list-modules` to see available modules."
                        );
                    }
                    std::process::exit(1);
                }
            }
            result
        }
    } else {
        omnifetch::default_modules()
    };

    if cfg.network {
        if !mods.iter().any(|m| m.id() == "publicip") {
            mods.push(Box::new(omnifetch::modules::publicip::PublicIp));
        }
        if !mods.iter().any(|m| m.id() == "weather") {
            mods.push(Box::new(omnifetch::modules::weather::Weather));
        }
    }

    if cfg.git && !mods.iter().any(|m| m.id() == "git") {
        mods.push(Box::new(omnifetch::modules::git::Git));
    }

    mods
}

fn pick_logo(cfg: &Config, cache: &mut StaticCache, dirty: &mut bool) -> Option<String> {
    match &cfg.logo {
        Logo::None => None,
        Logo::Mini => {
            if let Some(ref l) = cache.mini_logo {
                Some(l.clone())
            } else {
                let detected = detect("mini");
                cache.mini_logo = detected.clone();
                *dirty = true;
                detected
            }
        }
        Logo::Auto | Logo::Image(_) => {
            if let Some(ref l) = cache.normal_logo {
                Some(l.clone())
            } else {
                let detected = detect("normal");
                cache.normal_logo = detected.clone();
                *dirty = true;
                detected
            }
        }
        Logo::Named(k) => {
            let set = if k == "mini" { "mini" } else { "normal" };
            if let Some(art) = logo::lookup(k, set).or_else(|| logo::lookup(k, "mini")) {
                Some(art.to_string())
            } else {
                eprintln!("omnifetch: unknown logo '{k}'.");
                std::process::exit(1);
            }
        }
    }
}

fn detect(set: &str) -> Option<String> {
    let r = sys::os_release();
    let key = logo::normalize_key(r.id()).or_else(|| logo::normalize_key(&r.name()))?;
    logo::get(set, &key).map(str::to_string)
}
