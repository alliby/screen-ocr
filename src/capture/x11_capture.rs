use anyhow::Result;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use vello::kurbo::Rect;
use vello::peniko::Blob;
use x11rb::connection::Connection;
use x11rb::protocol::xproto;

fn screen_rect(rect: Rect) -> Result<Blob<u8>> {
    let rect = rect.into();
    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;
    let reply = xproto::get_image(
        &conn,
        xproto::ImageFormat::Z_PIXMAP,
        root,
        rect.x as _,
        rect.y as _,
        rect.width as _,
        rect.height as _,
        u32::MAX,
    )?
    .reply()?;

    let data = reply.data;
    let mut bytes = vec![(rect.width * rect.heigth) as usize * 4].into_boxed_slice();
    for chunck in data.chunks(4) {
        let pixel = [chunk[2], chunk[1], chunk[0], 255];
        bytes.write(&pixel[..])?;
    }

    Ok(Arc::from(bytes.into()))
}
