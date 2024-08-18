use anyhow::Result;
use std::sync::Arc;
use vello::kurbo::Rect;
use vello::peniko::Blob;
use windows::Win32::{Foundation::*, Graphics::Gdi::*};

pub fn screen_rect(rect: Rect) -> Result<Blob<u8>> {
    unsafe {
        // Get the device context handle of the screen
        let h_screen = GetDC(HWND(0));
        let h_dc = CreateCompatibleDC(h_screen);

        // Create bitmap handle
        let (x, y, w, h) = (
            rect.min_x() as i32,
            rect.min_y() as i32,
            rect.width().abs() as i32,
            rect.height().abs() as i32,
        );
        let h_bitmap = CreateCompatibleBitmap(h_screen, w, h);

        // Select the bitmap into the DC
        let h_old = SelectObject(h_dc, h_bitmap);

        // Copy the screen content into the bitmap
        BitBlt(h_dc, 0, 0, w, h, h_screen, x, y, SRCCOPY)?;

        // Create an rgba8 buffer to store the screenshot
        let mut buffer = vec![0; (w * h * 4) as usize].into_boxed_slice();

        // Copy bitmap data into the ImageBuffer
        for y in 0..h {
            for x in 0..w {
                let pixel = GetPixel(h_dc, x, y).0;
                let r = ((pixel) & 0xFF) as u8;
                let g = ((pixel >> 8) & 0xFF) as u8;
                let b = ((pixel >> 16) & 0xFF) as u8;
                let stride = (x + y * w) as usize * 4;
                buffer[stride..(stride + 4)].copy_from_slice(&[r, g, b, 255]);
            }
        }

        // Clean up
        SelectObject(h_dc, h_old);

	// The ok here is for returning Result from BOOL type
        DeleteObject(h_bitmap).ok()?;
        DeleteDC(h_dc).ok()?;
	
        ReleaseDC(HWND(0), h_screen);
        Ok(Blob::new(Arc::new(buffer)))
    }
}
