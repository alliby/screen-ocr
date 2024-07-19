use crate::helpers::Rectangle;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use x11rb::connection::Connection;
use x11rb::protocol::xproto;

fn screen_rect(rect: impl Into<Rectangle>) -> Result<(), Box<dyn Error>> {
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
    let mut pixels = Vec::with_capacity((rect.width * rect.heigth) as usize * 3);
    for chunck in data.chunks(4) {
        let bytes = [chunk[2], chunk[1], chunk[0]];
        pixels.write(&bytes[..])?;
    }

    Ok(pixels)
}
