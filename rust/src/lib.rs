mod bindings;
mod coders;

struct ImageInfo(*const bindings::ImageInfo);
struct Image(*mut bindings::Image);
struct ExceptionInfo(*mut bindings::ExceptionInfo);

#[macro_export]
macro_rules! register_coder {
    ($name:ident, $decoder:ident, $encoder:ident) => {
        paste::item! {
            #[no_mangle]
            pub extern "C" fn [<Register $name Image>]() -> libc::size_t {
                unsafe extern "C" fn decode(image_info: *const $crate::bindings::ImageInfo, exception: *mut $crate::bindings::ExceptionInfo) -> *mut $crate::bindings::Image {
                    unimplemented!()
                }

                unsafe extern "C" fn encode(image_info: *const $crate::bindings::ImageInfo, image: *mut $crate::bindings::Image, exception: *mut $crate::bindings::ExceptionInfo) -> $crate::bindings::MagickBooleanType {
                    let image_info = $crate::ImageInfo(image_info);
                    let mut image = $crate::Image(image);
                    let mut exception_info = $crate::ExceptionInfo(exception);
                    let result = $encoder(&image_info, &mut image, &mut exception_info);
                    match result {
                        Ok(()) => $crate::bindings::MagickTrue,
                        // TODO: do something with exception info in the Err
                        // case
                        Err(()) => $crate::bindings::MagickFalse,
                    }
                }

                let name = concat!(stringify!($name), "\0");
                unsafe {
                    let mut entry = $crate::bindings::AcquireMagickInfo(name.as_ptr().cast(), name.as_ptr().cast(), name.as_ptr().cast());
                    (*entry).decoder = Some(decode);
                    (*entry).encoder = Some(encode);
                    $crate::bindings::RegisterMagickInfo(entry);
                }
                $crate::bindings::MagickImageCoderSignature
            }

            #[no_mangle]
            pub extern "C" fn [<Unregister $name Image>]() {
                let name = concat!(stringify!($name), "\0");
                unsafe {
                    $crate::bindings::UnregisterMagickInfo(name.as_ptr().cast());
                }
            }
        }
    }
}
