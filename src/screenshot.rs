use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use dioxus::desktop::window;
use webkit2gtk::WebViewExt;
use wry::WebViewExtUnix;

/// Capture a screenshot of the current webview as a base64-encoded webp string.
pub async fn capture_screenshot() -> Result<String, String> {
    let desktop = window();
    let wk_webview = desktop.webview.webview();

    let surface = capture_surface(&wk_webview).await?;
    let webp_bytes = surface_to_webp(surface)?;
    Ok(STANDARD.encode(&webp_bytes))
}

/// Capture a screenshot and save directly to a webp file.
/// The image is scaled to match the logical window size (not device pixels).
pub async fn screenshot_to_file(path: &str) -> Result<(), String> {
    let desktop = window();
    let wk_webview = desktop.webview.webview();
    let logical_w = desktop.window.inner_size().width as f64
        / desktop.window.scale_factor();
    let logical_h = desktop.window.inner_size().height as f64
        / desktop.window.scale_factor();

    let surface = capture_surface(&wk_webview).await?;
    let webp_bytes = surface_to_webp_scaled(surface, logical_w as u32, logical_h as u32)?;
    std::fs::write(path, &webp_bytes).map_err(|e| format!("write failed: {e}"))?;
    Ok(())
}

async fn capture_surface(
    wk: &webkit2gtk::WebView,
) -> Result<cairo::ImageSurface, String> {
    use webkit2gtk::{SnapshotOptions, SnapshotRegion};

    let surface = wk
        .snapshot_future(SnapshotRegion::FullDocument, SnapshotOptions::NONE)
        .await
        .map_err(|e| format!("webkit snapshot failed: {e}"))?;

    unsafe {
        let raw = surface.to_raw_none();
        let w = cairo::ffi::cairo_image_surface_get_width(raw);
        let h = cairo::ffi::cairo_image_surface_get_height(raw);
        if w <= 0 || h <= 0 {
            return Err(format!("Invalid surface dimensions: {w}x{h}"));
        }
        let img = cairo::ImageSurface::create(cairo::Format::ARgb32, w, h)
            .map_err(|e| format!("create image surface: {e}"))?;
        let cr = cairo::Context::new(&img).map_err(|e| format!("cairo context: {e}"))?;
        cr.set_source_surface(&surface, 0.0, 0.0)
            .map_err(|e| format!("set source: {e}"))?;
        cr.paint().map_err(|e| format!("paint: {e}"))?;
        drop(cr);
        img.flush();
        Ok(img)
    }
}

fn surface_to_webp_scaled(
    surface: cairo::ImageSurface,
    target_w: u32,
    target_h: u32,
) -> Result<Vec<u8>, String> {
    let src_w = surface.width() as f64;
    let src_h = surface.height() as f64;
    let tw = target_w as f64;
    let th = target_h as f64;

    // If already the right size, skip scaling
    if (src_w - tw).abs() < 1.0 && (src_h - th).abs() < 1.0 {
        return surface_to_webp(surface);
    }

    let scaled = cairo::ImageSurface::create(cairo::Format::ARgb32, target_w as i32, target_h as i32)
        .map_err(|e| format!("create scaled surface: {e}"))?;
    let cr = cairo::Context::new(&scaled).map_err(|e| format!("cairo context: {e}"))?;
    cr.scale(tw / src_w, th / src_h);
    cr.set_source_surface(&surface, 0.0, 0.0)
        .map_err(|e| format!("set source: {e}"))?;
    cr.paint().map_err(|e| format!("paint: {e}"))?;
    drop(cr);
    scaled.flush();
    surface_to_webp(scaled)
}

fn surface_to_webp(mut surface: cairo::ImageSurface) -> Result<Vec<u8>, String> {
    let w = surface.width() as u32;
    let h = surface.height() as u32;
    let stride = surface.stride() as usize;
    let data = surface.data().map_err(|e| format!("surface data: {e}"))?;

    // Cairo ARGB32 on little-endian: bytes [B, G, R, A], premultiplied.
    // Convert to non-premultiplied RGBA for webp.
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for y in 0..h as usize {
        for x in 0..w as usize {
            let src = y * stride + x * 4;
            let dst = (y * w as usize + x) * 4;
            let b = data[src];
            let g = data[src + 1];
            let r = data[src + 2];
            let a = data[src + 3];
            if a == 0 {
                rgba[dst..dst + 4].copy_from_slice(&[0, 0, 0, 0]);
            } else {
                rgba[dst] = ((r as u16 * 255) / a as u16).min(255) as u8;
                rgba[dst + 1] = ((g as u16 * 255) / a as u16).min(255) as u8;
                rgba[dst + 2] = ((b as u16 * 255) / a as u16).min(255) as u8;
                rgba[dst + 3] = a;
            }
        }
    }

    let encoder = webp::Encoder::from_rgba(&rgba, w, h);
    let webp_data = encoder.encode(90.0);
    Ok(webp_data.to_vec())
}
