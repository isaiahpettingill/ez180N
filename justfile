set shell := ["sh", "-cu"]

# Override the configuration location when RetroArch uses a non-standard location.
retroarch_config := env_var_or_default("RETROARCH_CONFIG", env_var("HOME") + "/.config/retroarch/retroarch.cfg")

# Build and install the core and its metadata into the directories configured by RetroArch.
# RETROARCH_CORE_DIR and RETROARCH_INFO_DIR override their respective config values.
install:
    #!/usr/bin/env sh
    set -eu
    config='{{retroarch_config}}'
    test -f "$config" || { echo "RetroArch configuration not found: $config" >&2; exit 1; }
    core_dir=${RETROARCH_CORE_DIR:-$(sed -n 's/^[[:space:]]*libretro_directory[[:space:]]*=[[:space:]]*"\(.*\)"[[:space:]]*$/\1/p' "$config" | sed -n '1p')}
    info_dir=${RETROARCH_INFO_DIR:-$(sed -n 's/^[[:space:]]*libretro_info_path[[:space:]]*=[[:space:]]*"\(.*\)"[[:space:]]*$/\1/p' "$config" | sed -n '1p')}
    expand_home() { case "$1" in "~") printf '%s\n' "$HOME" ;; "~/"*) printf '%s/%s\n' "$HOME" "${1#\~/}" ;; *) printf '%s\n' "$1" ;; esac; }
    core_dir=$(expand_home "$core_dir")
    info_dir=$(expand_home "$info_dir")
    test -n "$core_dir" && test -d "$core_dir" || { echo "RetroArch core directory not found: $core_dir" >&2; exit 1; }
    test -n "$info_dir" && test -d "$info_dir" || { echo "RetroArch core info directory not found: $info_dir" >&2; exit 1; }
    cargo build --release
    install -m 755 target/release/libez180n.so "$core_dir/ez180n_libretro.so"
    install -m 644 ez180n_libretro.info "$info_dir/ez180n_libretro.info"
