use std::{fs, path::Path};

const APP_ID: &str = "io.github.zhubby.Shogun";
const LINUX_ICON_SIZES: &[u32] = &[16, 32, 48, 64, 128, 256, 512, 1024];

#[test]
fn packaging_assets_reference_platform_icons() {
    let info_plist = fs::read_to_string("packaging/macos/Info.plist").unwrap();
    assert!(info_plist.contains("Shogun.icns"));
    assert!(info_plist.contains(APP_ID));
    assert!(Path::new("packaging/macos/Shogun.icns").is_file());
    assert!(Path::new("packaging/windows/Shogun.ico").is_file());
    assert_eq!(
        &fs::read("packaging/macos/Shogun.icns").unwrap()[0..4],
        b"icns"
    );

    let windows_icon = fs::read("packaging/windows/Shogun.ico").unwrap();
    assert_eq!(&windows_icon[0..4], &[0, 0, 1, 0]);
    assert_eq!(u16::from_le_bytes([windows_icon[4], windows_icon[5]]), 4);

    let desktop = fs::read_to_string("packaging/linux/io.github.zhubby.Shogun.desktop").unwrap();
    assert!(desktop.contains("Exec=shogun"));
    assert!(desktop.contains(&format!("Icon={APP_ID}")));

    for size in LINUX_ICON_SIZES {
        let icon_path = format!("packaging/linux/hicolor/{size}x{size}/apps/{APP_ID}.png");
        assert!(Path::new(&icon_path).is_file(), "missing {icon_path}");
    }
}

#[test]
fn makefile_exposes_icon_and_package_targets() {
    let makefile = fs::read_to_string("Makefile").unwrap();

    for target in [
        "icons:",
        "icons-macos:",
        "icons-linux:",
        "icons-windows:",
        "package:",
        "package-macos:",
        "package-linux:",
        "package-windows:",
    ] {
        assert!(makefile.contains(target), "missing make target {target}");
    }

    assert!(makefile.contains("packaging/macos/$(APP_NAME).icns"));
    assert!(makefile.contains("packaging/windows/$(APP_NAME).ico"));
    assert!(makefile.contains("libexec/$(BIN_NAME)"));
    assert!(makefile.contains("x86_64-unknown-linux-gnu"));
    assert!(makefile.contains("x86_64-pc-windows-msvc"));
}
