#include "wrapper.h"

void rust_SetPixelViaPixelInfo(const Image *magick_restrict image,
    const PixelInfo *magick_restrict pixel_info,
    Quantum *magick_restrict pixel) {

    SetPixelViaPixelInfo(image, pixel_info, pixel);
}

size_t rust_GetPixelChannels(const Image *magick_restrict image) {
	return GetPixelChannels(image);
}