// fn read_wbmp_image(image_info: &magick::ImageInfo, exception_info: &magick::ExceptionInfo) -> Result<magick::Image, ()> {
//  let image = image_info.acquire_image(exception_info)?;
//  image.open_blob(magick::BlobMode::ReadBinary, exception_info)?;

//  let header = image.read_u16_le()?;
//  if header != 0 {
//      exception_info.throw(magick::Error::CoderError, "OnlyLevelZerofilesSupported")?;
//  }

//  let columns = read_wbmp_integer(image)?;
//  let rows = read_wbmp_integer(image)?;
//  if rows == 0 || columns == 0 {
//      exception_info.throw(magick::Error::CorruptImageError, "ImproperImageHeader")?;
//  }

//  if image.ping() {
//      image.close_blob();
//      return Ok(image.get_first_in_list());
//  }

//  image.set_extent(columns, rows, exception_info)?;

//  image.set_background_color();

//  for y in 0..rows {
//      let mut q = image.queue_authentic_pixels(0, y, columns, 1, exception_info)?;
//      let bits_remaining = 0;
//      let byte = 0;
//      for x in 0..columns {
//          if bits_remaining == 0 {
//              byte = image.read_byte()?;
//              bits_remaining = 8;
//          }
//          let pixel_val = byte & (0x1 << (bits_remaining - 1)) == 0 {
//              0
//          } else {
//              1
//          };
//          q.set_pixel(pixel_val);
//          bits_remaining -= 1;
//          q.advance(1);
//      }
//      image.sync_authentic_pixels(exception_info)?;
//      image.set_progress(magick::LoadImageTag, y, rows)?;
//  }
//  image.sync(exception_info);
//  image.close_blob();
//  Ok(image.get_first_in_list())
// }

#[allow(
    clippy::all,
    dead_code,
    non_camel_case_types,
    non_upper_case_globals,
    non_snake_case,
    improper_ctypes
)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

struct ImageInfo;
struct Image;
struct ExceptionInfo;

macro_rules! register_coder {
    ($name:ident, $decoder:ident, $encoder:ident) => {
        paste::item! {
            #[no_mangle]
            pub extern "C" fn [<Register $name Image>]() -> libc::size_t {
                unsafe extern "C" fn decode(image_info: *const bindings::ImageInfo, exception: *mut bindings::ExceptionInfo) -> *mut bindings::Image {
                    unimplemented!()
                }

                unsafe extern "C" fn encode(image_info: *const bindings::ImageInfo, image: *mut bindings::Image, exception: *mut bindings::ExceptionInfo) -> bindings::MagickBooleanType {
                    unimplemented!()
                }

                let name = concat!(stringify!($name), "\0");
                unsafe {
                    let mut entry = bindings::AcquireMagickInfo(name.as_ptr().cast(), name.as_ptr().cast(), name.as_ptr().cast());
                    // XXX: these need to be wrappers, not the Rust functions.
                    (*entry).decoder = Some(decode);
                    (*entry).encoder = Some(encode);
                    bindings::RegisterMagickInfo(entry);
                }
                bindings::BindingsMagickImageCoderSignature
            }

            #[no_mangle]
            pub extern "C" fn [<Unregister $name Image>]() {
                let name = concat!(stringify!($name), "\0");
                unsafe {
                    bindings::UnregisterMagickInfo(name.as_ptr().cast());
                }
            }
        }
    }
}

fn read_rust_image(image_info: &ImageInfo, exception_info: &ExceptionInfo) -> Result<Image, ()> {
    unimplemented!()
}

fn write_rust_image(
    image_info: &ImageInfo,
    image: &mut Image,
    exception_info: &ExceptionInfo,
) -> Result<(), ()> {
    unimplemented!()
}

register_coder!(RUST, read_rust_image, write_rust_image);
