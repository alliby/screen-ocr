use anyhow::Result;
use std::sync::Arc;
use vello::kurbo::Rect;
use vello::peniko::Blob;
use x11rb::connection::Connection;
use x11rb::protocol::xproto;
use xproto::ImageFormat;

pub fn screen_rect(rect: Rect) -> Result<Blob<u8>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;
    let (x, y, w, h) = (
        rect.min_x() as i16,
        rect.min_y() as i16,
        rect.width().abs() as u16,
        rect.height().abs() as u16,
    );
    let reply = xproto::get_image(&conn, ImageFormat::Z_PIXMAP, root, x, y, w, h, u32::MAX)?.reply()?;

    let data = reply.data.into_boxed_slice();
    Ok(Blob::new(Arc::new(data)))
}
