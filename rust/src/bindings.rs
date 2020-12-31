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

pub(crate) use bindings::*;

#[allow(non_upper_case_globals)]
pub const MagickImageCoderSignature: libc::size_t = BindingsMagickImageCoderSignature;
