use anyhow::Result;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use vello::kurbo::Rect;
use vello::peniko::Blob;
use x11rb::connection::Connection;
use x11rb::protocol::xproto;
use xproto::ImageFormat::Z_PIXMAP;

fn screen_rect(rect: Rect) -> Result<Blob<u8>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;
    let (x, y, w, h) = (
        rect.min_x() as i16,
        rect.min_y() as i16,
        rect.width().abs() as u16,
        rect.height().abs() as u16,
    );
    let reply = xproto::get_image(&conn, Z_PIXMAP, root, x, y, w, h, u32::MAX)?.reply()?;

    let data = reply.data;
    let mut bytes = vec![(rect.width * rect.heigth) as usize * 4].into_boxed_slice();
    for chunck in data.chunks(4) {
        let pixel = [chunk[2], chunk[1], chunk[0], 255];
        bytes.write(&pixel[..])?;
    }

    Ok(Arc::from(bytes.into()))
}
