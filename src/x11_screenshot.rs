use std::error::Error;
use std::fs::File;
use std::io::Write;
use x11rb::connection::Connection;
use x11rb::protocol::xproto;

fn main() -> Result<(), Box<dyn Error>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;
    let root_width = screen.width_in_pixels as isize;
    let root_height = screen.height_in_pixels as isize;
    let reply = xproto::get_image(
        &conn,
        xproto::ImageFormat::Z_PIXMAP,
        root,
        0,
        0,
        root_width as u16,
        root_height as u16,
        u32::MAX
    )?
    .reply()?;

    let data = reply.data;
    let mut f = File::create("output.ppm")?;
    write!(&mut f, "P6\n{} {} 255\n", root_width, root_height)?;
    for chunk in data.chunks(4) {
        let bytes = [chunk[2], chunk[1], chunk[0]];
        f.write(&bytes[..])?;
    }
    
    // let mut pixels = Vec::with_capacity(data.len());
    // for chunck in data.chunks(4) {
        
    // }
    Ok(())
}
