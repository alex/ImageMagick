fn read_null_image(image_info: &ImageInfo, exception_info: &ExceptionInfo) -> Result<Image, ()> {
    unimplemented!()
}

fn write_null_image(
    image_info: &ImageInfo,
    image: &mut Image,
    exception_info: &ExceptionInfo,
) -> Result<(), ()> {
    unimplemented!()
}

crate::register_coder!(NULL, read_rust_image, write_rust_image);
