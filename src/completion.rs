pub fn generate_bash() -> String {
    let mut logos = String::from("auto mini none");
    for (k, _) in crate::logo::NORMAL {
        logos.push(' ');
        logos.push_str(k);
    }

    format!(
        r#"# Bash completion for omnifetch
_omnifetch() {{
    local cur prev opts presets logos themes gradients
    COMPREPLY=()
    cur="${{COMP_WORDS[COMP_CWORD]}}"
    prev="${{COMP_WORDS[COMP_CWORD-1]}}"

    opts="-h --help -V --version -f --fast -a --all -j --json -c --config -m --modules -l --logo --logo-mini --logo-top --border --border-title --border-align --nerd --nerd-only -t --theme --list-themes --list-presets --gradient -i --image --image-cols --image-rows -p --preset -T --timing --no-cache --no-color --gen-config --gen-config-force --list-modules --git --network --completion --quotes-file --quotes --export"
    presets="minimal compact detailed modern hardware fastfetch neofetch paleofetch catnap macchina sysprint nitch pfetch all"
    logos="{logos}"
    themes="catppuccin tokyo-night nord gruvbox dracula rose-pine"
    gradients="rainbow sunset cyberpunk synthwave fire ice matrix dracula"

    case "${{prev}}" in
        --export)
            COMPREPLY=( $(compgen -W "svg html" -- "${{cur}}") )
            return 0
            ;;
        --border-title|--border-align)
            COMPREPLY=( $(compgen -W "left center right" -- "${{cur}}") )
            return 0
            ;;
        -p|--preset)
            COMPREPLY=( $(compgen -W "${{presets}}" -- "${{cur}}") )
            return 0
            ;;
        -l|--logo)
            COMPREPLY=( $(compgen -W "${{logos}}" -- "${{cur}}") )
            return 0
            ;;
        -t|--theme)
            COMPREPLY=( $(compgen -W "${{themes}}" -- "${{cur}}") )
            return 0
            ;;
        --gradient)
            COMPREPLY=( $(compgen -W "${{gradients}}" -- "${{cur}}") )
            return 0
            ;;
        --completion)
            COMPREPLY=( $(compgen -W "bash zsh fish" -- "${{cur}}") )
            return 0
            ;;
        -c|--config|-i|--image|--quotes-file|--quotes)
            COMPREPLY=( $(compgen -f -- "${{cur}}") )
            return 0
            ;;
        *)
            ;;
    esac

    if [[ "${{cur}}" == -* ]]; then
        COMPREPLY=( $(compgen -W "${{opts}}" -- "${{cur}}") )
        return 0
    fi
}}

complete -F _omnifetch omnifetch
"#
    )
}

pub fn generate_zsh() -> String {
    let mut logos_str = String::from(
        "        'auto:Auto-detect distro logo'\n        'mini:Compact mini ASCII logo'\n        'none:Disable logo'\n",
    );
    for (k, _) in crate::logo::NORMAL {
        logos_str.push_str(&format!("        '{k}:{k} logo'\n"));
    }

    format!(
        r#"#compdef omnifetch

_omnifetch() {{
    local -a presets logos shells
    presets=(
        'minimal:Ultra-fast minimalist display'
        'compact:Compact and dense layout with zero extra blank lines'
        'detailed:Everything at once, in full'
        'modern:Modern layout with CPU temperatures and network I/O'
        'hardware:Comprehensive hardware diagnostics'
        'fastfetch:Same modules as fastfetch'
        'neofetch:Same modules as Neofetch 7.x'
        'paleofetch:Same modules as paleofetch'
        'catnap:Same modules as catnap'
        'macchina:Same modules as macchina'
        'sysprint:Same modules as sysprint'
        'nitch:Same modules as nitch'
        'pfetch:Same modules as pfetch'
        'all:Run all available local modules'
    )
    logos=(
{logos_str}    )
    shells=(
        'bash:Generate Bash completion'
        'zsh:Generate Zsh completion'
        'fish:Generate Fish completion'
    )

    _arguments \
        '(-h --help)'{{-h,--help}}'[Print help message]' \
        '(-V --version)'{{-V,--version}}'[Print version]' \
        '(-f --fast)'{{-f,--fast}}'[Minimal set of modules for highest speed]' \
        '(-a --all)'{{-a,--all}}'[Run all available system modules]' \
        '(-j --json)'{{-j,--json}}'[Print machine-readable JSON]' \
        '(-p --preset)'{{-p,--preset}}'[Use built-in layout preset]:preset:->presets' \
        '(-l --logo)'{{-l,--logo}}'[Set distro logo]:logo:->logos' \
        '--logo-mini[Use mini ASCII logo]' \
        '--logo-top[Render logo centered on top instead of left]' \
        '--border[Wrap output in decorative unicode border box]' \
        '--border-title[Border category title alignment]:alignment:(left center right)' \
        '--border-align[Border category title alignment]:alignment:(left center right)' \
        '--nerd[Prefix module keys with Nerd Font icons]' \
        '--nerd-only[Display only Nerd Font icons]' \
        '(-t --theme)'{{-t,--theme}}'[Apply color theme]' \
        '--list-themes[List available built-in themes]' \
        '--list-presets[List available layout presets]' \
        '--gradient[Apply color gradient]' \
        '--git[Display current git repository statistics]' \
        '--network[Include network-dependent modules (publicip, weather)]' \
        '(-m --modules)'{{-m,--modules}}'[Comma-separated module ids to run]:modules:' \
        '(-c --config)'{{-c,--config}}'[Load configuration from file]:config file:_files' \
        '(-i --image)'{{-i,--image}}'[Display graphic image using Kitty/Sixel]:image file:_files' \
        '--image-cols[Width of image in character cells]:columns:' \
        '--image-rows[Height of image in terminal lines]:rows:' \
        '(-T --timing)'{{-T,--timing}}'[Print per-module execution timings]' \
        '--no-cache[Disable caching completely (always fetch fresh data)]' \
        '--quotes-file[Custom quotes JSON or text file to load quotes from]:quotes file:_files' \
        '--export[Export output as SVG or HTML ("svg", "html", or filename)]:export target:' \
        '--no-color[Disable colors]' \
        '--gen-config[Print default documented config.toml]' \
        '--gen-config-force[Overwrite existing config file without confirmation prompt]' \
        '--list-modules[Print all module ids and exit]' \
        '--completion[Generate shell completion script]:shell:->shells' && return 0

    case $state in
        presets)
            _describe -t presets 'preset' presets
            ;;
        logos)
            _describe -t logos 'logo' logos
            ;;
        shells)
            _describe -t shells 'shell' shells
            ;;
    esac
}}

_omnifetch "$@"
"#
    )
}

pub fn generate_fish() -> String {
    let mut logos = String::from("auto mini none");
    for (k, _) in crate::logo::NORMAL {
        logos.push(' ');
        logos.push_str(k);
    }

    format!(
        r#"# Fish completion for omnifetch

complete -c omnifetch -s h -l help -d 'Print help message'
complete -c omnifetch -s V -l version -d 'Print version'
complete -c omnifetch -s f -l fast -d 'Minimal set of modules for highest speed'
complete -c omnifetch -s a -l all -d 'Run all available system modules'
complete -c omnifetch -s j -l json -d 'Print machine-readable JSON'
complete -c omnifetch -l export -d 'Export output as SVG or HTML ("svg", "html", or filename)'
complete -c omnifetch -s p -l preset -d 'Use built-in layout preset' -x -a 'minimal compact detailed modern hardware fastfetch neofetch paleofetch catnap macchina sysprint nitch pfetch all'
complete -c omnifetch -s l -l logo -d 'Set distro logo' -x -a '{logos}'
complete -c omnifetch -l logo-mini -d 'Use mini ASCII logo'
complete -c omnifetch -l logo-top -d 'Render logo centered on top'
complete -c omnifetch -l border -d 'Wrap output in decorative border box'
complete -c omnifetch -l border-title -d 'Border category title alignment' -x -a 'left center right'
complete -c omnifetch -l border-align -d 'Border category title alignment' -x -a 'left center right'
complete -c omnifetch -l nerd -d 'Prefix module keys with Nerd Font icons'
complete -c omnifetch -l nerd-only -d 'Display only Nerd Font icons'
complete -c omnifetch -s t -l theme -d 'Apply color theme'
complete -c omnifetch -l list-themes -d 'List available built-in themes'
complete -c omnifetch -l list-presets -d 'List available layout presets'
complete -c omnifetch -l gradient -d 'Apply color gradient'
complete -c omnifetch -l git -d 'Display current git repository statistics'
complete -c omnifetch -l network -d 'Include network-dependent modules (publicip, weather)'
complete -c omnifetch -s m -l modules -d 'Comma-separated module ids'
complete -c omnifetch -s c -l config -d 'Load configuration file' -r
complete -c omnifetch -s i -l image -d 'Display graphic image file' -r
complete -c omnifetch -l image-cols -d 'Width of image in character cells' -x
complete -c omnifetch -l image-rows -d 'Height of image in terminal lines' -x
complete -c omnifetch -l quotes-file -d 'Custom quotes JSON or text file' -r
complete -c omnifetch -s T -l timing -d 'Print execution timings'
complete -c omnifetch -l no-cache -d 'Disable caching completely'
complete -c omnifetch -l no-color -d 'Disable colors'
complete -c omnifetch -l gen-config -d 'Print default config template'
complete -c omnifetch -l gen-config-force -d 'Overwrite existing config file without confirmation prompt'
complete -c omnifetch -l list-modules -d 'Print all module ids and exit'
complete -c omnifetch -l completion -d 'Generate completion script' -x -a 'bash zsh fish'
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completions_non_empty() {
        let bash = generate_bash();
        assert!(bash.contains("_omnifetch"));
        assert!(bash.contains("--network"));
        assert!(bash.contains("--gen-config-force"));
        assert!(bash.contains("arch"));

        let zsh = generate_zsh();
        assert!(zsh.contains("#compdef omnifetch"));
        assert!(zsh.contains("--network"));
        assert!(zsh.contains("--gen-config-force"));
        assert!(zsh.contains("arch:arch logo"));

        let fish = generate_fish();
        assert!(fish.contains("complete -c omnifetch"));
        assert!(fish.contains("-l network"));
        assert!(fish.contains("-l gen-config-force"));
        assert!(fish.contains("-l image-cols"));
        assert!(fish.contains("-l image-rows"));
    }
}
