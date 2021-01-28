#include "wrapper.h"

void rust_SetPixelViaPixelInfo(const Image *magick_restrict image,
    const PixelInfo *magick_restrict pixel_info,
    Quantum *magick_restrict pixel) {

    SetPixelViaPixelInfo(image, pixel_info, pixel);
}

void rust_SetPixelIndex(const Image *magick_restrict image,
    const Quantum index, Quantum *magick_restrict pixel) {

	SetPixelIndex(image, index, pixel);
}

MagickRealType rust_GetPixelLuma(const Image *magick_restrict image,
    const Quantum *magick_restrict pixel) {

	return GetPixelLuma(image, pixel);
}

size_t rust_GetPixelChannels(const Image *magick_restrict image) {
	return GetPixelChannels(image);
}