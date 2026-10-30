use super::model::TrayProxyMode;
use tauri::image::Image;

type Rgba = [u8; 4];

const SYSTEM_ACCENT: Rgba = [220, 38, 56, 255];
const TUN_ACCENT: Rgba = [168, 85, 247, 255];

pub fn recolor_icon_for_mode(base_icon: &Image<'_>, mode: TrayProxyMode) -> Option<Image<'static>> {
    let accent = match mode {
        TrayProxyMode::System => SYSTEM_ACCENT,
        TrayProxyMode::Tun => TUN_ACCENT,
        TrayProxyMode::Manual => return None,
    };

    let mut recolored = Vec::with_capacity(base_icon.rgba().len());
    // `as_chunks` 自 Rust 1.88 稳定（与本 crate 的 MSRV 一致），语义与 `chunks_exact` 相同
    let (pixels, _remainder) = base_icon.rgba().as_chunks::<4>();
    for pixel in pixels {
        let alpha = pixel[3];
        if alpha == 0 {
            recolored.extend_from_slice(pixel);
            continue;
        }

        // 保留原图明暗层次，只替换色相，避免换成完全陌生的托盘形状。
        let luminance =
            (u16::from(pixel[0]) * 54 + u16::from(pixel[1]) * 183 + u16::from(pixel[2]) * 19) / 256;
        let tinted = map_luminance_to_accent(accent, luminance as u8);

        recolored.extend_from_slice(&tinted);
        recolored.push(alpha);
    }

    Some(Image::new_owned(
        recolored,
        base_icon.width(),
        base_icon.height(),
    ))
}

fn map_luminance_to_accent(accent: Rgba, luminance: u8) -> [u8; 3] {
    let palette = palette_for_accent(accent);
    let base = palette.base;

    if luminance < 96 {
        let ratio = luminance as f32 / 96.0;
        mix_rgb(palette.shadow, base, ratio)
    } else {
        let ratio = ((luminance - 96) as f32 / 159.0).clamp(0.0, 1.0);
        mix_rgb(base, palette.highlight, ratio)
    }
}

fn palette_for_accent(accent: Rgba) -> TintPalette {
    if accent == SYSTEM_ACCENT {
        return TintPalette {
            base: [220, 38, 56],
            shadow: [92, 16, 28],
            highlight: [244, 96, 112],
        };
    }

    if accent == TUN_ACCENT {
        return TintPalette {
            base: [168, 85, 247],
            shadow: [72, 32, 116],
            highlight: [206, 156, 255],
        };
    }

    let base = rgb(accent);
    TintPalette {
        base,
        shadow: mix_rgb([24, 18, 30], base, 0.4),
        highlight: mix_rgb(base, [255, 255, 255], 0.18),
    }
}

struct TintPalette {
    base: [u8; 3],
    shadow: [u8; 3],
    highlight: [u8; 3],
}

fn rgb(color: Rgba) -> [u8; 3] {
    [color[0], color[1], color[2]]
}

fn mix_rgb(from: [u8; 3], to: [u8; 3], ratio: f32) -> [u8; 3] {
    [
        mix_channel(from[0], to[0], ratio),
        mix_channel(from[1], to[1], ratio),
        mix_channel(from[2], to[2], ratio),
    ]
}

fn mix_channel(from: u8, to: u8, ratio: f32) -> u8 {
    let clamped = ratio.clamp(0.0, 1.0);
    (from as f32 + (to as f32 - from as f32) * clamped).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_icon() -> Image<'static> {
        Image::new_owned(
            vec![
                255, 255, 255, 255, 80, 80, 80, 255, 0, 0, 0, 0, 200, 60, 30, 200,
            ],
            2,
            2,
        )
    }

    #[test]
    fn manual_mode_keeps_original_icon() {
        assert!(recolor_icon_for_mode(&sample_icon(), TrayProxyMode::Manual).is_none());
    }

    #[test]
    fn recolored_icon_keeps_size_and_alpha() {
        let base = sample_icon();
        let recolored = recolor_icon_for_mode(&base, TrayProxyMode::System).expect("system icon");

        assert_eq!(recolored.width(), 2);
        assert_eq!(recolored.height(), 2);
        assert_eq!(recolored.rgba()[3], 255);
        assert_eq!(recolored.rgba()[7], 255);
        assert_eq!(recolored.rgba()[11], 0);
        assert_eq!(recolored.rgba()[15], 200);
    }

    #[test]
    fn different_modes_produce_different_tints() {
        let base = sample_icon();
        let system = recolor_icon_for_mode(&base, TrayProxyMode::System).expect("system icon");
        let tun = recolor_icon_for_mode(&base, TrayProxyMode::Tun).expect("tun icon");

        assert_ne!(system.rgba(), base.rgba());
        assert_ne!(tun.rgba(), base.rgba());
        assert_ne!(system.rgba(), tun.rgba());
        assert!(system.rgba()[0] > system.rgba()[1]);
        assert!(system.rgba()[0] > system.rgba()[2]);
        assert!(tun.rgba()[0] > tun.rgba()[1]);
        assert!(tun.rgba()[2] > tun.rgba()[1]);
    }
}
