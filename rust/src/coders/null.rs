fn read_null_image(
    image_info: &crate::ImageInfo,
    exception_info: &mut crate::ExceptionInfo,
) -> Result<crate::Image, ()> {
    let mut image = image_info.acquire_image(exception_info)?;
    let cols = if image.columns() == 0 {
        1
    } else {
        image.columns()
    };
    let rows = if image.rows() == 0 { 1 } else { image.rows() };
    let background_color = image.background_color();

    let (mut canvas, _) =
        image.open_blob_read(image_info, crate::BlobMode::Binary, exception_info)?;
    canvas.set_alpha_trait(crate::PixelTrait::Blend);
    canvas.set_extent(cols, rows, exception_info)?;
    let mut background = canvas.conform_pixel_info(&background_color, exception_info)?;
    background.set_alpha(crate::TRANSPARENT_ALPHA);

    for y in 0..rows {
        let mut q = canvas.queue_authentic_pixels(0, y, cols, 1, exception_info)?;
        for x in 0..cols {
            q.set_pixel_from_info(x, &background);
        }
        canvas.sync_authentic_pixels(exception_info)?;
    }
    Ok(image)
}

/// `write_null_image` writes no output at all. It is useful when specified
/// as an output format when profiling.
fn write_null_image(
    _image_info: &crate::ImageInfo,
    _image: &mut crate::Image,
    _exception_info: &mut crate::ExceptionInfo,
) -> Result<(), ()> {
    Ok(())
}

crate::register_coder!(
    NULL,
    "Constant image of uniform color",
    read_null_image,
    write_null_image,
    crate::CoderFlags::DEFAULT - crate::CoderFlags::ADJOIN,
    crate::FormatType::Implicit
);
