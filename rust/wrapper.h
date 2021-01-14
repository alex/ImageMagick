#include "MagickCore/studio.h"
#include "MagickCore/blob.h"
#include "MagickCore/blob-private.h"
#include "MagickCore/cache.h"
#include "MagickCore/color.h"
#include "MagickCore/color-private.h"
#include "MagickCore/colorspace-private.h"
#include "MagickCore/exception.h"
#include "MagickCore/exception-private.h"
#include "MagickCore/image.h"
#include "MagickCore/image-private.h"
#include "MagickCore/list.h"
#include "MagickCore/magick.h"
#include "MagickCore/memory_.h"
#include "MagickCore/pixel-accessor.h"
#include "MagickCore/quantum-private.h"
#include "MagickCore/static.h"
#include "MagickCore/string_.h"
#include "MagickCore/module.h"

// Workaround bindgen limitation.
const size_t BindingsMagickImageCoderSignature = MagickImageCoderSignature;

const Quantum BindingsTransparentAlpha = TransparentAlpha;

// Definitions for static inline functions from ImageMagick that we re-export
// in wrapper.c
void rust_SetPixelViaPixelInfo(const Image *magick_restrict image,
    const PixelInfo *magick_restrict pixel_info,
    Quantum *magick_restrict pixel);
