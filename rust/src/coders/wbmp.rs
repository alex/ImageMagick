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
