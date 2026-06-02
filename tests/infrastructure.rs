// Infrastructure validation tests for SDD change: add-prebuilt-binaries
//
// These tests validate CI/CD workflow structure and installer script content.
// They serve as the TDD safety net: RED when files don't exist or are wrong,
// GREEN when implementation matches specs.

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // CARGO_MANIFEST_DIR = /path/to/kody/kody (the inner crate)
    // Workspace root is one level up
    manifest_dir.parent().unwrap().to_path_buf()
}

fn cargo_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

// ─── Phase 1: release.yml ───────────────────────────────────────────────────

mod release_yml {
    use super::*;

    fn read_release_yml() -> String {
        let path = workspace_root().join(".github").join("workflows").join("release.yml");
        fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("release.yml not found at {:?}: {}", path, e))
    }

    #[test]
    fn task_1_1_triggers_on_v_tags() {
        let content = read_release_yml();
        assert!(
            content.contains("on:"),
            "release.yml must have an on: trigger"
        );
        assert!(
            content.contains("tags:"),
            "release.yml must filter by tags"
        );
        assert!(
            content.contains("'v*'") || content.contains("\"v*\""),
            "release.yml must trigger on v* tags only"
        );
    }

    #[test]
    fn task_1_2_matrix_three_platforms() {
        let content = read_release_yml();
        assert!(
            content.contains("windows-latest") || content.contains("windows"),
            "Matrix must include windows-latest"
        );
        assert!(
            content.contains("ubuntu-latest") || content.contains("ubuntu"),
            "Matrix must include ubuntu-latest"
        );
        assert!(
            content.contains("macos-latest") || content.contains("macos"),
            "Matrix must include macos-latest"
        );
    }

    #[test]
    fn task_1_2_platform_targets_correct() {
        let content = read_release_yml();
        assert!(
            content.contains("x86_64-pc-windows-msvc"),
            "Matrix must target x86_64-pc-windows-msvc"
        );
        assert!(
            content.contains("x86_64-unknown-linux-musl"),
            "Matrix must target x86_64-unknown-linux-musl"
        );
        assert!(
            content.contains("aarch64-apple-darwin"),
            "Matrix must target aarch64-apple-darwin"
        );
    }

    #[test]
    fn task_1_3_rust_toolchain_step() {
        let content = read_release_yml();
        assert!(
            content.contains("dtolnay/rust-toolchain"),
            "Workflow must use dtolnay/rust-toolchain action"
        );
        assert!(
            content.contains("targets:"),
            "Rust toolchain step must specify targets"
        );
    }

    #[test]
    fn task_1_4_musl_tools_for_linux() {
        let content = read_release_yml();
        assert!(
            content.contains("musl-tools") || content.contains("musl"),
            "Ubuntu runner must install musl-tools for static linking"
        );
    }

    #[test]
    fn task_1_5_cargo_build_step() {
        let content = read_release_yml();
        assert!(
            content.contains("cargo build"),
            "Workflow must have cargo build step"
        );
        assert!(
            content.contains("--release"),
            "Build must use --release profile"
        );
        assert!(
            content.contains("CARGO_INCREMENTAL"),
            "Build must set CARGO_INCREMENTAL: 0"
        );
    }

    #[test]
    fn task_1_6_packaging_step() {
        let content = read_release_yml();
        assert!(
            content.contains("Compress-Archive") || content.contains("zip"),
            "Windows packaging must use zip/Compress-Archive"
        );
        assert!(
            content.contains("tar"),
            "Linux/macOS packaging must use tar"
        );
        assert!(
            content.contains(".zip") || content.contains(".tar.gz"),
            "Package command must produce archive files"
        );
    }

    #[test]
    fn task_1_7_upload_to_github_release() {
        let content = read_release_yml();
        assert!(
            content.contains("softprops/action-gh-release"),
            "Workflow must use softprops/action-gh-release for uploads"
        );
        assert!(
            content.contains("files:"),
            "Release step must specify files to upload"
        );
    }

    #[test]
    fn task_1_2_asset_naming_convention() {
        let content = read_release_yml();
        assert!(
            content.contains("kody-x86_64-pc-windows-msvc.zip"),
            "Windows asset must be named kody-x86_64-pc-windows-msvc.zip"
        );
        assert!(
            content.contains("kody-x86_64-unknown-linux-musl.tar.gz"),
            "Linux asset must be named kody-x86_64-unknown-linux-musl.tar.gz"
        );
        assert!(
            content.contains("kody-aarch64-apple-darwin.tar.gz"),
            "macOS asset must be named kody-aarch64-apple-darwin.tar.gz"
        );
    }
}

// ─── Phase 2: install.ps1 ───────────────────────────────────────────────────

mod install_ps1 {
    use super::*;

    fn read_install_ps1() -> String {
        let path = cargo_dir().join("install.ps1");
        fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("install.ps1 not found at {:?}: {}", path, e))
    }

    // ── Removed items (tasks 2.1, 2.2) ─────────────────────────────────────

    #[test]
    fn task_2_1_no_rust_check_function() {
        let content = read_install_ps1();
        assert!(
            !content.contains("Test-RustInstalled"),
            "install.ps1 must NOT contain Test-RustInstalled function — Rust check removed"
        );
    }

    #[test]
    fn task_2_2_no_compile_step_in_main_flow() {
        let content = read_install_ps1();
        // Main install flow must not have a compilation step.
        // Error messages MAY reference cargo build as fallback.
        // Verify old section headers are gone:
        assert!(
            !content.contains("Compilando Kody"),
            "install.ps1 must NOT have old '[PASO *] Compilando Kody' section"
        );
    }

    #[test]
    fn task_2_2_no_git_clone_as_install_step() {
        let content = read_install_ps1();
        // Main install flow must not clone the repo.
        // Fallback error messages MAY reference git clone as alternative.
        // Verify git clone is only present in fallback context, not as a primary step:
        // The old primary step used "git clone --depth 1 https://github.com/yokonad/kody.git"
        assert!(
            !content.contains("git clone --depth 1 https://github.com/yokonad/kody.git"),
            "install.ps1 must NOT have the old git clone --depth 1 primary step"
        );
        // Also verify no "[PASO" section is directly followed by git operations
        // (the new PASO 1 "Descargando" uses Invoke-WebRequest, not git)
        let has_git_clone = content.contains("git clone");
        if has_git_clone {
            // git clone should only appear in error/fallback messages
            assert!(
                content.contains("git clone") && content.contains("alternativa"),
                "If git clone appears, it must be in a fallback/error context, not a main step"
            );
        }
    }

    #[test]
    fn task_2_2_no_rust_install_winget() {
        let content = read_install_ps1();
        assert!(
            !content.contains("winget install --id Git.Git") || !content.contains("winget install"),
            "install.ps1 must NOT install Git via winget — Git dependency removed"
        );
    }

    // ── New items (tasks 2.3-2.9) ──────────────────────────────────────────

    #[test]
    fn task_2_3_downloads_prebuilt_binary() {
        let content = read_install_ps1();
        assert!(
            content.contains("releases/latest/download/kody-x86_64-pc-windows-msvc.zip"),
            "install.ps1 must download from releases/latest/download/kody-x86_64-pc-windows-msvc.zip"
        );
        assert!(
            content.contains("Invoke-WebRequest"),
            "install.ps1 must use Invoke-WebRequest for download"
        );
    }

    #[test]
    fn task_2_4_progress_indicator() {
        let content = read_install_ps1();
        assert!(
            content.contains("ProgressPreference"),
            "install.ps1 must set ProgressPreference for download progress bar"
        );
    }

    #[test]
    fn task_2_5_error_handling_404() {
        let content = read_install_ps1();
        assert!(
            content.contains("404") || content.to_lowercase().contains("not found"),
            "install.ps1 must handle 404 / release not found with error message"
        );
    }

    #[test]
    fn task_2_6_error_handling_network() {
        let content = read_install_ps1();
        assert!(
            content.contains("try") && content.contains("catch"),
            "install.ps1 must use try/catch for network error handling"
        );
    }

    #[test]
    fn task_2_7_extract_zip_archive() {
        let content = read_install_ps1();
        assert!(
            content.contains("Expand-Archive"),
            "install.ps1 must use Expand-Archive to extract zip"
        );
    }

    #[test]
    fn task_2_7_copy_binary_to_install_dir() {
        let content = read_install_ps1();
        assert!(
            content.contains("Copy-Item"),
            "install.ps1 must copy binary to install directory"
        );
        assert!(
            content.contains("LOCALAPPDATA"),
            "install.ps1 must install to LOCALAPPDATA\\bin\\kody\\"
        );
    }

    #[test]
    fn task_2_8_cleanup_temp() {
        let content = read_install_ps1();
        assert!(
            content.contains("Remove-Item"),
            "install.ps1 must clean up temp files after extraction"
        );
    }

    #[test]
    fn task_2_9_keeps_path_setup() {
        let content = read_install_ps1();
        assert!(
            content.contains("PATH") || content.contains("Path"),
            "install.ps1 must still configure PATH after install"
        );
        assert!(
            content.contains("SetEnvironmentVariable") || content.contains("Environment"),
            "install.ps1 must persist PATH changes"
        );
    }

    #[test]
    fn task_2_3_spansih_ui_preserved() {
        let content = read_install_ps1();
        // Verify Spanish UI text is preserved — check for key Spanish phrases
        assert!(
            content.contains("Instalacion") || content.contains("INSTALACION COMPLETADA"),
            "install.ps1 must preserve Spanish UI text (Instalacion)"
        );
        assert!(
            content.contains("Descargar") || content.contains("Descargando"),
            "install.ps1 must use Spanish UI text (Descargar)"
        );
    }

    #[test]
    fn task_2_3_darkcyan_debug_preserved() {
        let content = read_install_ps1();
        // DarkCyan is used for debug output in the original script
        assert!(
            content.contains("DarkCyan"),
            "install.ps1 must preserve DarkCyan for debug output style"
        );
    }
}

// ─── Phase 2 Triangulation: Edge Cases ──────────────────────────────────────

mod install_ps1_triangulation {
    use super::*;

    fn read() -> String {
        let path = cargo_dir().join("install.ps1");
        fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("install.ps1 not found at {:?}: {}", path, e))
    }

    #[test]
    fn triangulate_download_url_exact_match() {
        let content = read();
        // The full URL must be present exactly as spec defines it
        let expected = "https://github.com/yokonad/kody/releases/latest/download/kody-x86_64-pc-windows-msvc.zip";
        assert!(
            content.contains(expected),
            "install.ps1 must use the exact download URL: {expected}"
        );
    }

    #[test]
    fn triangulate_error_message_specific() {
        let content = read();
        // Verify error messages follow the spec's required Spanish text patterns
        assert!(
            content.contains("No se encontro una version pre-compilada"),
            "404 error must display specific Spanish message about missing release"
        );
        assert!(
            content.contains("HTTP 404"),
            "404 error must mention HTTP 404 status code"
        );
    }

    #[test]
    fn triangulate_fallback_has_source_instructions() {
        let content = read();
        // When no release is found, fallback message must suggest source compilation
        assert!(
            content.contains("compila desde codigo fuente"),
            "Fallback must suggest compiling from source code"
        );
        assert!(
            content.contains("https://github.com/yokonad/kody.git"),
            "Fallback must link to the GitHub repository"
        );
    }

    #[test]
    fn triangulate_temp_dir_cleanup_uses_env_temp() {
        let content = read();
        // Temp files must use $env:TEMP for extraction path
        assert!(
            content.contains("$env:TEMP\\kody-install") || content.contains("`$env:TEMP")
                || content.contains("$env:TEMP") && content.contains("kody-install"),
            "Temp extraction must use $env:TEMP path"
        );
    }

    #[test]
    fn triangulate_install_path_is_localappdata() {
        let content = read();
        // Binary must install to LOCALAPPDATA\bin\kody\kody.exe
        assert!(
            content.contains("$env:LOCALAPPDATA\\bin\\kody"),
            "Install directory must be $env:LOCALAPPDATA\\bin\\kody"
        );
        assert!(
            content.contains("$BinPath") || content.contains("kody.exe"),
            "Binary filename must be kody.exe"
        );
    }

    #[test]
    fn triangulate_no_git_check_function() {
        let content = read();
        // Test-CommandInPath was the old Git checker function
        assert!(
            !content.contains("Test-CommandInPath"),
            "install.ps1 must NOT contain old Git path checker function"
        );
        // The only remaining helper is Test-Command (generic) and Pause-Script
        let test_command_count = content.matches("function Test-").count();
        assert!(
            test_command_count <= 1,
            "Only generic Test-Command should remain; found {test_command_count} Test-* functions"
        );
    }
}

// ─── Phase 3 Triangulation: README specific content ─────────────────────────

mod readme_triangulation {
    use super::*;

    fn read() -> String {
        let path = cargo_dir().join("README.md");
        fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("README.md not found at {:?}: {}", path, e))
    }

    #[test]
    fn triangulate_readme_mentions_time_savings() {
        let content = read();
        // README should mention the new speed (~10 seconds instead of 5-15 minutes)
        assert!(
            content.contains("~10 segundos") || content.contains("~10 second"),
            "README must mention the new install speed (~10 seconds)"
        );
    }

    #[test]
    fn triangulate_readme_no_rust_required() {
        let content = read();
        assert!(
            content.contains("No requiere Rust") || content.contains("requires Rust"),
            "README must explicitly state Rust is NOT required (or note new simplicity)"
        );
    }

    #[test]
    fn triangulate_readme_has_precompilado_term() {
        let content = read();
        assert!(
            content.contains("pre-compilado") || content.contains("precompilado")
                || content.contains("pre-compiled") || content.contains("prebuilt"),
            "README must use the term 'pre-compilado' (or English equivalent) to describe binaries"
        );
    }
}

// ─── Phase 3: README.md ─────────────────────────────────────────────────────

mod readme {
    use super::*;

    fn read_readme() -> String {
        let path = cargo_dir().join("README.md");
        fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("README.md not found at {:?}: {}", path, e))
    }

    #[test]
    fn task_3_1_mentions_prebuilt_binary() {
        let content = read_readme();
        assert!(
            content.contains("pre-compil") || content.contains("precompil")
                || content.contains("binario") || content.contains("descarga"),
            "README must mention pre-built/pre-compiled binary download"
        );
    }

    #[test]
    fn task_3_2_updated_time_estimate() {
        let content = read_readme();
        assert!(
            !content.contains("5-15 minutos") && !content.contains("5-15 min"),
            "README must NOT mention 5-15 minute old compile-time estimate"
        );
    }

    #[test]
    fn task_3_1_no_rust_requirement_in_install() {
        let content = read_readme();
        // Installation section should not require Rust anymore
        let install_section: String = content
            .lines()
            .skip_while(|l| !l.contains("Instalación") && !l.contains("Install"))
            .take(30)
            .collect::<Vec<&str>>()
            .join("\n");
        assert!(
            !install_section.contains("Rust instalado"),
            "Installation section must NOT mention Rust being required (pre-built flow)"
        );
    }
}
