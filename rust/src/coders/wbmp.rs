fn read_wbmp_integer(
    blob: &mut crate::BlobReader,
    exception_info: &mut crate::ExceptionInfo,
) -> Result<usize, ()> {
    let mut value = 0usize;
    loop {
        let byte = blob.read_u8(exception_info)?;
        value <<= 7;
        value |= (byte as usize) & 0x7f;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
    }
}

fn read_wbmp_image(
    image_info: &crate::ImageInfo,
    exception_info: &mut crate::ExceptionInfo,
) -> Result<crate::Image, ()> {
    let mut image = image_info.acquire_image(exception_info)?;
    let (mut canvas, mut blob) =
        image.open_blob_read(image_info, crate::BlobMode::Binary, exception_info)?;

    let header = blob.read_u16_be(exception_info)?;
    if header != 0 {
        crate::throw!(
            exception_info,
            crate::Exception::CoderError,
            "OnlyLevelZerofilesSupported"
        )?;
    }

    let columns = read_wbmp_integer(&mut blob, exception_info)?;
    let rows = read_wbmp_integer(&mut blob, exception_info)?;
    if rows == 0 || columns == 0 {
        crate::throw!(
            exception_info,
            crate::Exception::CorruptImageError,
            "ImproperImageHeader"
        )?;
    }

    if canvas.is_ping() {
        drop(blob);
        return Ok(image);
    }

    canvas.set_extent(columns, rows, exception_info)?;

    canvas.set_pixels_to_background_color(exception_info)?;

    canvas.acquire_colormap(2, exception_info)?;

    for y in 0..rows {
        let mut q = canvas.queue_authentic_pixels(0, y, columns, 1, exception_info)?;
        let mut bits_remaining = 0;
        let mut byte = 0;
        for x in 0..columns {
            if bits_remaining == 0 {
                byte = blob.read_u8(exception_info)?;
                bits_remaining = 8;
            }
            let pixel_val = if byte & (0x1 << (bits_remaining - 1)) == 0 {
                0.
            } else {
                1.
            };
            q.set_pixel_index(x, pixel_val);
            bits_remaining -= 1;
        }
        drop(q);
        canvas.sync_authentic_pixels(exception_info)?;
        canvas.set_progress(y, rows);
    }
    canvas.sync(exception_info)?;
    drop(blob);
    Ok(image)
}

fn write_wbmp_integer(blob: &mut crate::BlobWriter, mut v: usize) {
    let mut buf = [0u8; 5];
    let mut i = buf.len() - 1;
    let mut first = true;
    while v > 0 {
        let mut octet = (v & 0x7f) as u8;
        v >>= 7;
        if !first {
            octet |= 0x80;
        }
        buf[i] = octet;
        i -= 1;
        first = false;
    }
    blob.write_all(&buf[i + 1..])
}

fn write_wbmp_image(
    image_info: &crate::ImageInfo,
    image: &mut crate::Image,
    exception_info: &mut crate::ExceptionInfo,
) -> Result<(), ()> {
    let columns = image.columns();
    let rows = image.rows();

    let (mut canvas, mut blob) =
        image.open_blob_write(image_info, crate::BlobMode::Binary, exception_info)?;
    canvas.transform_colorspace(crate::Colorspace::SRGB, exception_info)?;
    canvas.set_type(crate::ImageType::Bilevel, exception_info)?;

    blob.write_u16_be(0);
    write_wbmp_integer(&mut blob, columns);
    write_wbmp_integer(&mut blob, rows);

    for y in 0..rows {
        let p = canvas.get_virtual_pixels(0, y, columns, 1, exception_info)?;
        let mut bit = 0;
        let mut byte = 0;
        for x in 0..columns {
            if p.get_pixel_luma(x) >= (crate::QUANTUM_RANGE / 2.).into() {
                byte |= 0x1 << (7 - bit);
            }
            bit += 1;
            if bit == 8 {
                blob.write_u8(byte);
                bit = 0;
                byte = 0;
            }
        }
        if bit != 0 {
            blob.write_u8(byte);
        }
        canvas.set_progress(y, rows);
    }

    Ok(())
}

crate::register_coder!(
    WBMP,
    "Wireless Bitmap (level 0) image",
    read_wbmp_image,
    write_wbmp_image,
    crate::CoderFlags::DEFAULT - crate::CoderFlags::ADJOIN,
    crate::FormatType::Undefined
);
