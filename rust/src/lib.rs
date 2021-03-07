use std::convert::TryInto;
use std::ffi::CString;
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

    pub(crate) fn background_color(&self) -> PixelInfo {
        PixelInfo(unsafe { (*self.0).background_color })
    }

    pub(crate) fn open_blob_read<'a>(
        &'a mut self,
        image_info: &ImageInfo,
        mode: BlobMode,
        exception_info: &mut ExceptionInfo,
    ) -> Result<(CanvasWriter<'a>, BlobReader<'a>), ()> {
        let c_mode = match mode {
            BlobMode::Binary => bindings::BlobMode_ReadBinaryBlobMode,
        };
        let status = unsafe { bindings::OpenBlob(image_info.0, self.0, c_mode, exception_info.0) };
        exception_info.check()?;
        if status == bindings::MagickBooleanType_MagickFalse {
            return Err(());
        }
        let ptr = self.0;
        Ok((
            CanvasWriter(ptr, std::marker::PhantomData),
            BlobReader(ptr, std::marker::PhantomData),
        ))
    }

    pub(crate) fn open_blob_write<'a>(
        &'a mut self,
        image_info: &ImageInfo,
        mode: BlobMode,
        exception_info: &mut ExceptionInfo,
    ) -> Result<(CanvasReader<'a>, BlobWriter<'a>), ()> {
        let c_mode = match mode {
            BlobMode::Binary => bindings::BlobMode_WriteBinaryBlobMode,
        };
        let status = unsafe { bindings::OpenBlob(image_info.0, self.0, c_mode, exception_info.0) };
        exception_info.check()?;
        if status == bindings::MagickBooleanType_MagickFalse {
            return Err(());
        }
        let ptr = self.0;
        Ok((
            CanvasReader(ptr, std::marker::PhantomData),
            BlobWriter(ptr, std::marker::PhantomData),
        ))
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            bindings::DestroyImageList(self.0);
        }
    }
}

struct BlobReader<'a>(*mut bindings::Image, std::marker::PhantomData<&'a ()>);

impl BlobReader<'_> {
    fn read(&mut self, data: &mut [u8], exception_info: &mut ExceptionInfo) -> Result<(), ()> {
        let read = unsafe { bindings::ReadBlob(self.0, data.len(), data.as_mut_ptr().cast()) };
        if read != data.len().try_into().unwrap() {
            throw!(exception_info, Exception::CorruptImageError, "ShortRead")?;
        }
        Ok(())
    }

    pub(crate) fn read_u8(&mut self, exception_info: &mut ExceptionInfo) -> Result<u8, ()> {
        let mut data = [0; 1];
        self.read(&mut data, exception_info)?;
        Ok(u8::from_le_bytes(data))
    }

    pub(crate) fn read_u16_be(&mut self, exception_info: &mut ExceptionInfo) -> Result<u16, ()> {
        let mut data = [0; 2];
        self.read(&mut data, exception_info)?;
        Ok(u16::from_be_bytes(data))
    }
}

impl Drop for BlobReader<'_> {
    fn drop(&mut self) {
        unsafe {
            bindings::CloseBlob(self.0);
        }
    }
}

struct BlobWriter<'a>(*mut bindings::Image, std::marker::PhantomData<&'a ()>);

impl BlobWriter<'_> {
    fn write_u8(&mut self, v: u8) {
        unsafe {
            bindings::WriteBlobByte(self.0, v);
        }
    }

    fn write_u16_be(&mut self, v: u16) {
        unsafe {
            bindings::WriteBlobMSBShort(self.0, v);
        }
    }

    fn write_all(&mut self, buf: &[u8]) {
        unsafe {
            bindings::WriteBlob(self.0, buf.len(), buf.as_ptr().cast());
        }
    }
}

impl Drop for BlobWriter<'_> {
    fn drop(&mut self) {
        unsafe {
            bindings::CloseBlob(self.0);
        }
    }
}

struct CanvasWriter<'a>(*mut bindings::Image, std::marker::PhantomData<&'a ()>);

impl<'a> CanvasWriter<'a> {
    fn num_channels(&self) -> usize {
        unsafe { bindings::rust_GetPixelChannels(self.0) }
    }

    pub(crate) fn is_ping(&self) -> bool {
        unsafe { (*self.0).ping == bindings::MagickBooleanType_MagickTrue }
    }

    pub(crate) fn set_alpha_trait(&mut self, value: PixelTrait) {
        unsafe {
            (*self.0).alpha_trait = value as u32;
        }
    }

    pub(crate) fn set_progress(&mut self, offset: usize, extent: usize) {
        let result = unsafe {
            bindings::SetImageProgress(
                self.0,
                bindings::LoadImageTag.as_ptr().cast(),
                offset.try_into().unwrap(),
                extent.try_into().unwrap(),
            )
        };
        assert!(result == bindings::MagickBooleanType_MagickTrue);
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

    pub(crate) fn set_pixels_to_background_color(
        &mut self,
        exception_info: &mut ExceptionInfo,
    ) -> Result<(), ()> {
        let status = unsafe { bindings::SetImageBackgroundColor(self.0, exception_info.0) };
        exception_info.check()?;
        if status == bindings::MagickBooleanType_MagickFalse {
            return Err(());
        }
        Ok(())
    }

    pub(crate) fn acquire_colormap(
        &mut self,
        n_colors: usize,
        exception_info: &mut ExceptionInfo,
    ) -> Result<(), ()> {
        let status = unsafe { bindings::AcquireImageColormap(self.0, n_colors, exception_info.0) };
        exception_info.check()?;
        if status == bindings::MagickBooleanType_MagickFalse {
            return Err(());
        }
        Ok(())
    }

    pub(crate) fn queue_authentic_pixels<'b>(
        &'b mut self,
        x: usize,
        y: usize,
        columns: usize,
        rows: usize,
        exception_info: &mut ExceptionInfo,
    ) -> Result<AuthenticPixels<'a, 'b>, ()> {
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
            canvas: self,
            quantums: q,
            quantums_length: q_len,
        })
    }

    pub(crate) fn sync(&mut self, exception_info: &mut ExceptionInfo) -> Result<(), ()> {
        // No callers from C actually check the return code, so we don't
        // either.
        unsafe { bindings::SyncImage(self.0, exception_info.0) };
        exception_info.check()?;
        Ok(())
    }

    pub(crate) fn sync_authentic_pixels(
        &mut self,
        exception_info: &mut ExceptionInfo,
    ) -> Result<(), ()> {
        let status = unsafe { bindings::SyncAuthenticPixels(self.0, exception_info.0) };
        if status == bindings::MagickBooleanType_MagickFalse {
            return Err(());
        }
        exception_info.check()?;
        Ok(())
    }
}

struct CanvasReader<'a>(*mut bindings::Image, std::marker::PhantomData<&'a ()>);

impl<'a> CanvasReader<'a> {
    fn num_channels(&self) -> usize {
        unsafe { bindings::rust_GetPixelChannels(self.0) }
    }

    fn transform_colorspace(
        &mut self,
        colorspace: Colorspace,
        exception_info: &mut ExceptionInfo,
    ) -> Result<(), ()> {
        unsafe {
            bindings::TransformImageColorspace(self.0, colorspace as u32, exception_info.0);
        }
        exception_info.check()
    }

    fn set_type(
        &mut self,
        image_type: ImageType,
        exception_info: &mut ExceptionInfo,
    ) -> Result<(), ()> {
        unsafe {
            bindings::SetImageType(self.0, image_type as u32, exception_info.0);
        }
        exception_info.check()
    }

    pub(crate) fn get_virtual_pixels<'b>(
        &'b mut self,
        x: usize,
        y: usize,
        columns: usize,
        rows: usize,
        exception_info: &mut ExceptionInfo,
    ) -> Result<VirtualPixels<'a, 'b>, ()> {
        let p = unsafe {
            bindings::GetVirtualPixels(
                self.0,
                x.try_into().unwrap(),
                y.try_into().unwrap(),
                columns,
                rows,
                exception_info.0,
            )
        };
        exception_info.check()?;
        let p_len = columns
            .checked_mul(rows)
            .unwrap()
            .checked_mul(self.num_channels())
            .unwrap();
        Ok(VirtualPixels {
            canvas: self,
            quantums: p,
            quantums_length: p_len,
        })
    }

    pub(crate) fn set_progress(&mut self, offset: usize, extent: usize) {
        let result = unsafe {
            bindings::SetImageProgress(
                self.0,
                bindings::SaveImageTag.as_ptr().cast(),
                offset.try_into().unwrap(),
                extent.try_into().unwrap(),
            )
        };
        assert!(result == bindings::MagickBooleanType_MagickTrue);
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

pub struct Quantum(bindings::Quantum);

impl From<u8> for Quantum {
    fn from(v: u8) -> Quantum {
        Quantum(v as bindings::Quantum)
    }
}

impl Into<bindings::MagickRealType> for Quantum {
    fn into(self) -> bindings::MagickRealType {
        self.0.into()
    }
}

impl std::ops::Div<u8> for Quantum {
    type Output = Self;

    fn div(self, rhs: u8) -> Self::Output {
        Quantum(self.0 / (rhs as bindings::Quantum))
    }
}

struct PixelInfo(bindings::PixelInfo);

impl PixelInfo {
    fn set_alpha(&mut self, value: Quantum) {
        self.0.alpha = value.0.into();
    }
}

struct AuthenticPixels<'a, 'b> {
    canvas: &'b mut CanvasWriter<'a>,
    quantums: *mut bindings::Quantum,
    quantums_length: usize,
}

impl AuthenticPixels<'_, '_> {
    fn set_pixel_index(&mut self, idx: usize, index: Quantum) {
        let pos = self.canvas.num_channels().checked_mul(idx).unwrap();
        assert!(pos < self.quantums_length);
        unsafe {
            bindings::rust_SetPixelIndex(self.canvas.0, index.0, self.quantums.add(pos));
        }
    }

    fn set_pixel_from_info(&mut self, idx: usize, pixel: &PixelInfo) {
        let pos = self.canvas.num_channels().checked_mul(idx).unwrap();
        assert!(pos < self.quantums_length);
        unsafe {
            bindings::rust_SetPixelViaPixelInfo(self.canvas.0, &pixel.0, self.quantums.add(pos));
        }
    }
}

struct VirtualPixels<'a, 'b> {
    canvas: &'b mut CanvasReader<'a>,
    quantums: *const bindings::Quantum,
    quantums_length: usize,
}

impl VirtualPixels<'_, '_> {
    fn get_pixel_luma(&self, idx: usize) -> f64 {
        let pos = self.canvas.num_channels().checked_mul(idx).unwrap();
        assert!(pos < self.quantums_length);
        unsafe { bindings::rust_GetPixelLuma(self.canvas.0, self.quantums.add(pos)) }
    }
}

pub const TRANSPARENT_ALPHA: Quantum = Quantum(bindings::TransparentAlpha);
pub const QUANTUM_RANGE: Quantum = Quantum(bindings::QuantumRange as bindings::Quantum);

#[repr(u32)]
enum PixelTrait {
    Blend = bindings::PixelTrait_BlendPixelTrait,
}

#[repr(u32)]
enum FormatType {
    Undefined = bindings::MagickFormatType_UndefinedFormatType,
    Implicit = bindings::MagickFormatType_ImplicitFormatType,
}

enum BlobMode {
    Binary,
}

#[repr(u32)]
enum Exception {
    CoderError = bindings::ExceptionType_CoderError,
    CorruptImageError = bindings::ExceptionType_CorruptImageError,
}

#[repr(u32)]
enum Colorspace {
    SRGB = bindings::ColorspaceType_sRGBColorspace,
}

#[repr(u32)]
enum ImageType {
    Bilevel = bindings::ImageType_BilevelType,
}

pub(crate) fn _throw(
    exception_info: &mut ExceptionInfo,
    exc: Exception,
    tag: &'static str,
    file: &'static str,
    func: &'static str,
    line: u32,
) {
    let c_tag = CString::new(tag).unwrap();
    let c_file = CString::new(file).unwrap();
    let c_func = CString::new(func).unwrap();
    unsafe {
        bindings::ThrowMagickException(
            exception_info.0,
            c_file.as_ptr(),
            c_func.as_ptr(),
            line.try_into().unwrap(),
            exc as u32,
            c_tag.as_ptr(),
            "\0".as_ptr().cast(),
        );
    }
}

#[macro_export]
macro_rules! function {
    () => {{
        fn __placeholder() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(__placeholder);
        &name[..name.len() - "::__placeholder".len()]
    }};
}

#[macro_export]
macro_rules! throw {
    ($exception_info:expr, $exc:expr, $tag:expr) => {{
        $crate::_throw(
            $exception_info,
            $exc,
            $tag,
            file!(),
            $crate::function!(),
            line!(),
        );
        Err(())
    }};
}

#[macro_export]
macro_rules! register_coder {
    ($name:ident, $description:expr, $decoder:ident, $encoder:ident, $flags:expr, $format_type:expr) => {
        paste::item! {
            #[no_mangle]
            pub extern "C" fn [<Register $name Image>]() -> libc::size_t {
                unsafe extern "C" fn decode(image_info: *const $crate::bindings::ImageInfo, exception: *mut $crate::bindings::ExceptionInfo) -> *mut $crate::bindings::Image {
                    let image_info = $crate::ImageInfo(image_info);
                    let mut exception_info = $crate::ExceptionInfo(exception);
                    let result = $decoder(&image_info, &mut exception_info);
                    match result {
                        Ok(image) => {
                            let im = std::mem::ManuallyDrop::new(image);
                            im.0
                        },
                        Err(()) => {
                            assert!(exception_info.check().is_err());
                            std::ptr::null_mut()
                        },
                    }
                }

                unsafe extern "C" fn encode(image_info: *const $crate::bindings::ImageInfo, image: *mut $crate::bindings::Image, exception: *mut $crate::bindings::ExceptionInfo) -> $crate::bindings::MagickBooleanType {
                    let image_info = $crate::ImageInfo(image_info);
                    let mut image = std::mem::ManuallyDrop::new($crate::Image(image));
                    let mut exception_info = $crate::ExceptionInfo(exception);
                    let result = $encoder(&image_info, &mut image, &mut exception_info);
                    match result {
                        Ok(()) => $crate::bindings::MagickBooleanType_MagickTrue,
                        Err(()) => {
                            assert!(exception_info.check().is_err());
                            $crate::bindings::MagickBooleanType_MagickFalse
                        },
                    }
                }

                let name = concat!(stringify!($name), "\0");
                let description = concat!($description, "\0");
                unsafe {
                    let mut entry = $crate::bindings::AcquireMagickInfo(name.as_ptr().cast(), name.as_ptr().cast(), description.as_ptr().cast());
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
