use super::*;
use serde_json::json;

fn sample_releases() -> Vec<serde_json::Value> {
    vec![
        json!({
            "tag_name": "v1.3.0-autobuild",
            "name": "Autobuild Nightly",
            "prerelease": true
        }),
        json!({
            "tag_name": "v1.2.0-rc.1",
            "name": "Release Candidate",
            "prerelease": true
        }),
        json!({
            "tag_name": "v1.1.0",
            "name": "Stable Release",
            "prerelease": false
        }),
    ]
}

#[test]
fn update_channel_should_resolve_inputs_consistently() {
    assert_eq!(
        UpdateChannel::from_inputs(Some("stable"), true),
        UpdateChannel::Stable
    );
    assert_eq!(
        UpdateChannel::from_inputs(Some(" Prerelease "), false),
        UpdateChannel::Prerelease
    );
    assert_eq!(
        UpdateChannel::from_inputs(Some("autobuild"), false),
        UpdateChannel::Autobuild
    );
    assert_eq!(
        UpdateChannel::from_inputs(None, false),
        UpdateChannel::Stable
    );
    assert_eq!(
        UpdateChannel::from_inputs(Some("unknown"), true),
        UpdateChannel::Prerelease
    );
}

#[test]
fn update_channel_should_report_release_list_usage() {
    assert!(!UpdateChannel::Stable.uses_release_list());
    assert!(UpdateChannel::Prerelease.uses_release_list());
    assert!(UpdateChannel::Autobuild.uses_release_list());
}

#[test]
fn check_arch_compatibility_should_match_known_arch_aliases() {
    assert!(check_arch_compatibility(
        "sing-box-windows-amd64.exe",
        "x86_64"
    ));
    assert!(check_arch_compatibility("sing-box-windows.exe", "x86_64"));
    assert!(!check_arch_compatibility(
        "sing-box-windows-arm64.exe",
        "x86_64"
    ));

    assert!(check_arch_compatibility(
        "sing-box-macos-arm64.dmg",
        "aarch64"
    ));
    assert!(check_arch_compatibility(
        "sing-box-macos-universal.dmg",
        "aarch64"
    ));
    assert!(!check_arch_compatibility(
        "sing-box-macos-x64.dmg",
        "aarch64"
    ));

    assert!(check_arch_compatibility(
        "sing-box-linux-armv7.deb",
        "armv7"
    ));
    assert!(!check_arch_compatibility(
        "sing-box-linux-arm64.deb",
        "armv7"
    ));

    assert!(check_arch_compatibility("sing-box-linux-i386.deb", "x86"));
    assert!(!check_arch_compatibility("sing-box-linux-x64.deb", "x86"));
    assert!(check_arch_compatibility(
        "sing-box-linux-x86_64.rpm",
        "x86_64"
    ));
    assert!(!check_arch_compatibility(
        "sing-box-linux-aarch64.rpm",
        "x86_64"
    ));
}

#[test]
fn get_package_kind_should_detect_supported_linux_formats() {
    assert_eq!(
        get_package_kind("sing-box-linux-x86_64.rpm"),
        PackageKind::Rpm
    );
    assert_eq!(
        get_package_kind("sing-box-windows_2.2.6_amd64.AppImage"),
        PackageKind::AppImage
    );
    assert_eq!(
        get_package_kind("sing-box-windows_2.2.6_amd64.deb"),
        PackageKind::Deb
    );
}

#[test]
fn detect_linux_package_preference_should_follow_os_release() {
    let fedora_os_release = r#"
ID=fedora
ID_LIKE="fedora rhel"
"#;
    assert_eq!(
        detect_linux_package_preference_from_os_release(fedora_os_release),
        LinuxPackagePreference::Rpm
    );

    let ubuntu_os_release = r#"
ID=ubuntu
ID_LIKE=debian
"#;
    assert_eq!(
        detect_linux_package_preference_from_os_release(ubuntu_os_release),
        LinuxPackagePreference::Deb
    );

    let arch_os_release = r#"
ID=arch
ID_LIKE=archlinux
"#;
    assert_eq!(
        detect_linux_package_preference_from_os_release(arch_os_release),
        LinuxPackagePreference::AppImage
    );
}

#[test]
fn get_platform_priority_for_linux_should_respect_distribution_preference() {
    let rpm = "sing-box-windows-2.2.6-1.x86_64.rpm";
    let deb = "sing-box-windows_2.2.6_amd64.deb";
    let appimage = "sing-box-windows_2.2.6_amd64.AppImage";

    assert!(
        get_platform_priority_for(rpm, "linux", "x86_64", LinuxPackagePreference::Rpm)
            > get_platform_priority_for(deb, "linux", "x86_64", LinuxPackagePreference::Rpm)
    );
    assert!(
        get_platform_priority_for(deb, "linux", "x86_64", LinuxPackagePreference::Deb)
            > get_platform_priority_for(rpm, "linux", "x86_64", LinuxPackagePreference::Deb)
    );
    assert!(
        get_platform_priority_for(
            appimage,
            "linux",
            "x86_64",
            LinuxPackagePreference::AppImage
        ) > get_platform_priority_for(rpm, "linux", "x86_64", LinuxPackagePreference::AppImage)
    );
}

#[test]
fn resolve_update_filename_and_message_should_cover_rpm() {
    assert_eq!(
        resolve_update_filename("https://example.com/app-2.2.6-1.x86_64.rpm", "linux"),
        "update.rpm"
    );
    assert_eq!(
        resolve_install_message("linux", "https://example.com/app-2.2.6-1.x86_64.rpm"),
        "正在安装软件包，请根据提示输入密码..."
    );
}

#[test]
fn select_release_by_channel_should_pick_expected_release() {
    let releases = sample_releases();

    let stable = select_release_by_channel(&releases, UpdateChannel::Stable)
        .expect("stable channel should find a non-prerelease release");
    assert_eq!(stable["tag_name"].as_str(), Some("v1.1.0"));

    let prerelease = select_release_by_channel(&releases, UpdateChannel::Prerelease)
        .expect("prerelease channel should use the first release entry");
    assert_eq!(prerelease["tag_name"].as_str(), Some("v1.3.0-autobuild"));

    let autobuild = select_release_by_channel(&releases, UpdateChannel::Autobuild)
        .expect("autobuild channel should prefer autobuild-tagged releases");
    assert_eq!(autobuild["tag_name"].as_str(), Some("v1.3.0-autobuild"));
}

#[test]
fn autobuild_channel_should_fallback_to_prerelease_when_needed() {
    let releases = vec![
        json!({
            "tag_name": "v1.2.0-rc.1",
            "name": "Release Candidate",
            "prerelease": true
        }),
        json!({
            "tag_name": "v1.1.0",
            "name": "Stable Release",
            "prerelease": false
        }),
    ];

    let autobuild = select_release_by_channel(&releases, UpdateChannel::Autobuild)
        .expect("autobuild channel should fallback to a prerelease release");
    assert_eq!(autobuild["tag_name"].as_str(), Some("v1.2.0-rc.1"));
}

#[test]
fn compare_versions_should_handle_semver_and_plain_text_versions() {
    assert!(compare_versions("v1.0.0", "1.0.1"));
    assert!(compare_versions("1.0.0 build-1", "v1.1.0 latest"));
    assert!(!compare_versions("1.1.0", "1.1.0"));

    assert!(compare_versions("nightly-2026-01-01", "nightly-2026-01-02"));
    assert!(!compare_versions(
        "nightly-2026-01-01",
        "nightly-2026-01-01"
    ));
}

#[test]
fn supports_in_app_update_should_only_enable_windows() {
    assert!(supports_in_app_update_for_platform("windows"));
    assert!(!supports_in_app_update_for_platform("linux"));
    assert!(!supports_in_app_update_for_platform("macos"));
}

#[test]
fn resolve_release_page_url_should_prefer_html_url() {
    let release = json!({
        "html_url": "https://github.com/xinggaoya/sing-box-windows/releases/tag/v2.2.6"
    });
    assert_eq!(
        resolve_release_page_url(&release),
        "https://github.com/xinggaoya/sing-box-windows/releases/tag/v2.2.6"
    );

    let release_without_url = json!({});
    assert_eq!(
        resolve_release_page_url(&release_without_url),
        "https://github.com/xinggaoya/sing-box-windows/releases"
    );
}
