fn read_null_image(image_info: &crate::ImageInfo, exception_info: &crate::ExceptionInfo) -> Result<crate::Image, ()> {
    unimplemented!()
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

crate::register_coder!(NULL, read_null_image, write_null_image);
