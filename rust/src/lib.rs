use std::convert::TryInto;
use std::mem;

mod bindings;
mod coders;

bitflags::bitflags! {
    struct CoderFlags: u32 {
        const ADJOIN = bindings::MagickInfoFlag_CoderAdjoinFlag;
        const BLOB_SUPPORT = bindings::MagickInfoFlag_CoderBlobSupportFlag;
        const DECODER_THREAD_SUPPORT = bindings::MagickInfoFlag_CoderDecoderThreadSupportFlag;
        const ENCODER_THREAD_SUPPORT = bindings::MagickInfoFlag_CoderEncoderThreadSupportFlag;
        const USE_EXTENSION = bindings::MagickInfoFlag_CoderUseExtensionFlag;

        const DEFAULT = Self::ADJOIN.bits | Self::BLOB_SUPPORT.bits | Self::DECODER_THREAD_SUPPORT.bits | Self::ENCODER_THREAD_SUPPORT.bits | Self::USE_EXTENSION.bits;
    }
}

struct ImageInfo(*const bindings::ImageInfo);

impl ImageInfo {
    pub(crate) fn acquire_image(&self, exception_info: &mut ExceptionInfo) -> Result<Image, ()> {
        let image = unsafe { bindings::AcquireImage(self.0, exception_info.0) };
        exception_info.check()?;
        Ok(Image(image))
    }
}

struct Image(*mut bindings::Image);

impl Image {
    pub(crate) fn rows(&self) -> usize {
        unsafe { (*self.0).rows }
    }

    pub(crate) fn columns(&self) -> usize {
        unsafe { (*self.0).columns }
    }

    fn num_channels(&self) -> usize {
        // TODO: ought to be GetPixelChannel but it's `static inline`
        unsafe { (*self.0).number_channels }
    }

    pub(crate) fn background_color(&self) -> PixelInfo {
        PixelInfo(unsafe { (*self.0).background_color })
    }

    pub(crate) fn set_alpha_trait(&mut self, value: PixelTrait) {
        unsafe {
            (*self.0).alpha_trait = value as u32;
        }
    }

    pub(crate) fn set_extent(
        &mut self,
        cols: usize,
        rows: usize,
        exception_info: &mut ExceptionInfo,
    ) -> Result<(), ()> {
        let status = unsafe { bindings::SetImageExtent(self.0, cols, rows, exception_info.0) };
        if status == bindings::MagickBooleanType_MagickFalse {
            return Err(());
        }
        exception_info.check()
    }

    pub(crate) fn conform_pixel_info(
        &mut self,
        source: &PixelInfo,
        exception_info: &mut ExceptionInfo,
    ) -> Result<PixelInfo, ()> {
        let mut dst = mem::MaybeUninit::uninit();
        unsafe {
            bindings::ConformPixelInfo(self.0, &source.0, dst.as_mut_ptr(), exception_info.0);
        }
        exception_info.check()?;
        Ok(PixelInfo(unsafe { dst.assume_init() }))
    }

    pub(crate) fn queue_authentic_pixels(
        &mut self,
        x: usize,
        y: usize,
        columns: usize,
        rows: usize,
        exception_info: &mut ExceptionInfo,
    ) -> Result<AuthenticPixels, ()> {
        let q = unsafe {
            bindings::QueueAuthenticPixels(
                self.0,
                x.try_into().unwrap(),
                y.try_into().unwrap(),
                columns,
                rows,
                exception_info.0,
            )
        };
        exception_info.check()?;
        let q_len = columns
            .checked_mul(rows)
            .unwrap()
            .checked_mul(self.num_channels())
            .unwrap();
        Ok(AuthenticPixels {
            image: self,
            quantums: q,
            quantums_length: q_len,
        })
    }

    pub(crate) fn sync_authentic_pixels(
        &mut self,
        exception_info: &mut ExceptionInfo,
    ) -> Result<(), ()> {
        let status = unsafe {
            bindings::SyncAuthenticPixels(self.0, exception_info.0)
        };
        if status == bindings::MagickBooleanType_MagickFalse {
            return Err(());
        }
        exception_info.check()?;
        Ok(())
    }
}

struct ExceptionInfo(*mut bindings::ExceptionInfo);

impl ExceptionInfo {
    fn check(&self) -> Result<(), ()> {
        if unsafe { (*self.0).severity } == bindings::ExceptionType_UndefinedException {
            Ok(())
        } else {
            Err(())
        }
    }
}

type Quantum = bindings::Quantum;

struct PixelInfo(bindings::PixelInfo);

impl PixelInfo {
    fn set_alpha(&mut self, value: Quantum) {
        self.0.alpha = value.into();
    }
}

struct AuthenticPixels<'a> {
    image: &'a mut Image,
    quantums: *mut Quantum,
    quantums_length: usize,
}

impl AuthenticPixels<'_> {
    fn set_pixel_from_info(&mut self, idx: usize, pixel: &PixelInfo) {
        let pos = self.image.num_channels().checked_mul(idx).unwrap();
        assert!(pos < self.quantums_length);
        unsafe {
            bindings::rust_SetPixelViaPixelInfo(self.image.0, &pixel.0, self.quantums.add(pos));
        }
    }
}

pub const TRANSPARENT_ALPHA: Quantum = bindings::TransparentAlpha;

#[repr(u32)]
enum PixelTrait {
    Blend = bindings::PixelTrait_BlendPixelTrait,
}

#[repr(u32)]
enum FormatType {
    Implicit = bindings::MagickFormatType_ImplicitFormatType,
}

#[macro_export]
macro_rules! register_coder {
    ($name:ident, $decoder:ident, $encoder:ident, $flags:expr, $format_type:expr) => {
        paste::item! {
            #[no_mangle]
            pub extern "C" fn [<Register $name Image>]() -> libc::size_t {
                unsafe extern "C" fn decode(image_info: *const $crate::bindings::ImageInfo, exception: *mut $crate::bindings::ExceptionInfo) -> *mut $crate::bindings::Image {
                    let image_info = $crate::ImageInfo(image_info);
                    let mut exception_info = $crate::ExceptionInfo(exception);
                    let result = $decoder(&image_info, &mut exception_info);
                    match result {
                        Ok(image) => image.0,
                        // TODO: do something with exception info in the Err
                        // case
                        Err(()) => std::ptr::null_mut(),
                    }
                }

                unsafe extern "C" fn encode(image_info: *const $crate::bindings::ImageInfo, image: *mut $crate::bindings::Image, exception: *mut $crate::bindings::ExceptionInfo) -> $crate::bindings::MagickBooleanType {
                    let image_info = $crate::ImageInfo(image_info);
                    let mut image = $crate::Image(image);
                    let mut exception_info = $crate::ExceptionInfo(exception);
                    let result = $encoder(&image_info, &mut image, &mut exception_info);
                    match result {
                        Ok(()) => $crate::bindings::MagickBooleanType_MagickTrue,
                        // TODO: do something with exception info in the Err
                        // case
                        Err(()) => $crate::bindings::MagickBooleanType_MagickFalse,
                    }
                }

                let name = concat!(stringify!($name), "\0");
                unsafe {
                    let mut entry = $crate::bindings::AcquireMagickInfo(name.as_ptr().cast(), name.as_ptr().cast(), name.as_ptr().cast());
                    (*entry).decoder = Some(decode);
                    (*entry).encoder = Some(encode);
                    (*entry).flags = ($flags).bits;
                    (*entry).format_type = $format_type as $crate::bindings::MagickFormatType;
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
