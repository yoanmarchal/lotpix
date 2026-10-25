// Animated WebP encoding, frame by frame, with libwebp.
//
// The `webp` crate keeps every frame in memory and loses the duration of the last one: the
// libwebp animation encoder is therefore called directly.
use std::ffi::{CStr, c_int};
use std::mem::MaybeUninit;
use std::ptr;

use libwebp_sys as sys;

use crate::t;

pub struct AnimatedWebpEncoder {
    // Invariant: non-null pointer returned by `WebPAnimEncoderNewInternal`, freed in `drop`
    encoder: *mut sys::WebPAnimEncoder,
    config: sys::WebPConfig,
    width: u32,
    height: u32,
}

impl AnimatedWebpEncoder {
    /// `loop_count` is 0 for an infinite loop.
    pub fn new(width: u32, height: u32, quality: f32, loop_count: i32) -> Result<Self, String> {
        let mut config = sys::WebPConfig::new().map_err(|()| t!("error-webp-config"))?;
        config.quality = quality;

        let abi_version = sys::WebPGetMuxABIVersion();
        let mut options = MaybeUninit::<sys::WebPAnimEncoderOptions>::uninit();
        // SAFETY: `options` points to an area of the right size, which libwebp fully initializes
        // when the call succeeds; it is only read in that case.
        let mut options = unsafe {
            if sys::WebPAnimEncoderOptionsInitInternal(options.as_mut_ptr(), abi_version) == 0 {
                return Err(t!("error-webp-version"));
            }
            options.assume_init()
        };
        options.anim_params.loop_count = loop_count;

        // SAFETY: `options` is initialized and stays valid during the call, which copies it.
        let encoder = unsafe {
            sys::WebPAnimEncoderNewInternal(width as c_int, height as c_int, &options, abi_version)
        };
        if encoder.is_null() {
            return Err(t!("error-webp-animation"));
        }
        Ok(Self {
            encoder,
            config,
            width,
            height,
        })
    }

    /// Adds an RGBA frame with the animation dimensions, shown from `timestamp_ms`.
    pub fn add_frame(&mut self, rgba: &[u8], timestamp_ms: i32) -> Result<(), String> {
        let stride = self.width as usize * 4;
        if rgba.len() != stride * self.height as usize {
            return Err(t!("error-webp-dimensions"));
        }

        let mut picture = sys::WebPPicture::new().map_err(|()| t!("error-webp-version"))?;
        picture.use_argb = 1;
        picture.width = self.width as c_int;
        picture.height = self.height as c_int;

        // SAFETY: `rgba` holds exactly `height` rows of `stride` bytes (checked above). The
        // import copies the pixels into `picture`, adding copies `picture` into the encoder:
        // `picture` can be freed in every case right after.
        let is_added = unsafe {
            let is_added = sys::WebPPictureImportRGBA(&mut picture, rgba.as_ptr(), stride as c_int)
                != 0
                && sys::WebPAnimEncoderAdd(self.encoder, &mut picture, timestamp_ms, &self.config)
                    != 0;
            sys::WebPPictureFree(&mut picture);
            is_added
        };
        if is_added {
            Ok(())
        } else {
            Err(self.last_error())
        }
    }

    /// Ends the animation. `end_timestamp_ms` sets the duration of the last frame.
    pub fn finish(self, end_timestamp_ms: i32) -> Result<Vec<u8>, String> {
        let mut data = sys::WebPData {
            bytes: ptr::null(),
            size: 0,
        };
        // SAFETY: a null frame tells libwebp the animation ends. `data` receives a buffer
        // allocated by libwebp, copied then freed by `WebPDataClear` before returning.
        unsafe {
            if sys::WebPAnimEncoderAdd(self.encoder, ptr::null_mut(), end_timestamp_ms, ptr::null())
                == 0
                || sys::WebPAnimEncoderAssemble(self.encoder, &mut data) == 0
            {
                return Err(self.last_error());
            }
            let bytes = if data.bytes.is_null() {
                Vec::new()
            } else {
                std::slice::from_raw_parts(data.bytes, data.size).to_vec()
            };
            sys::WebPDataClear(&mut data);
            Ok(bytes)
        }
    }

    fn last_error(&self) -> String {
        // SAFETY: libwebp returns a zero-terminated C string, valid while the encoder exists;
        // it is copied before returning.
        unsafe {
            let message = sys::WebPAnimEncoderGetError(self.encoder);
            if message.is_null() {
                t!("error-webp-unknown")
            } else {
                CStr::from_ptr(message).to_string_lossy().into_owned()
            }
        }
    }
}

impl Drop for AnimatedWebpEncoder {
    fn drop(&mut self) {
        // SAFETY: `encoder` is non-null and is only freed here.
        unsafe { sys::WebPAnimEncoderDelete(self.encoder) }
    }
}
