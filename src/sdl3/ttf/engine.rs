use crate::{
    get_error,
    libc::c_int,
    pixels::Color,
    render::TextureCreator,
    ttf::{
        sys::{
            TTF_CreateRendererTextEngine, TTF_CreateText, TTF_DestroyRendererTextEngine,
            TTF_DestroyText, TTF_DrawRendererText, TTF_GetTextSize, TTF_SetTextColor,
            TTF_SetTextFont, TTF_SetTextString, TTF_SetTextWrapWidth, TTF_Text, TTF_TextEngine,
            TTF_UpdateText,
        },
        Font,
    },
    Error,
};
use std::ffi::CString;

pub struct TextEngine {
    raw: *mut TTF_TextEngine,
}
impl TextEngine {
    #[doc(alias = "TTF_CreateRendererTextEngine")]
    pub fn new<T>(creator: &TextureCreator<T>) -> Result<Self, Error> {
        let raw = unsafe { TTF_CreateRendererTextEngine(creator.raw()) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Self { raw })
        }
    }

    pub fn raw(&self) -> *mut TTF_TextEngine {
        self.raw
    }

    #[doc(alias = "TTF_CreateText")]
    pub fn create_text(&self, font: &Font, text: &str) -> Result<Text, Error> {
        let ctext = CString::new(text).unwrap();
        let raw =
            unsafe { TTF_CreateText(self.raw, font.raw(), ctext.as_ptr(), ctext.count_bytes()) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Text { raw })
        }
    }
}
impl Drop for TextEngine {
    fn drop(&mut self) {
        unsafe { TTF_DestroyRendererTextEngine(self.raw) };
    }
}

pub struct Text {
    raw: *mut TTF_Text,
}
impl Text {
    pub fn raw(&self) -> *mut TTF_Text {
        self.raw
    }

    #[doc(alias = "TTF_UpdateText")]
    pub fn update(&mut self) -> Result<(), Error> {
        let ok = unsafe { TTF_UpdateText(self.raw) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "TTF_DrawRendererText")]
    pub fn draw(&self, x: f32, y: f32) -> Result<(), Error> {
        let ok = unsafe { TTF_DrawRendererText(self.raw, x, y) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "TTF_GetTextSize")]
    pub fn size(&self) -> (u32, u32) {
        let mut w: c_int = 0;
        let mut h: c_int = 0;
        unsafe { TTF_GetTextSize(self.raw, &mut w, &mut h) };
        (w as u32, h as u32)
    }

    #[doc(alias = "TTF_SetTextFont")]
    pub fn set_font(&mut self, font: &Font) -> Result<(), Error> {
        let ok = unsafe { TTF_SetTextFont(self.raw, font.raw()) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "TTF_SetTextString")]
    pub fn set_text(&mut self, text: &str) -> Result<(), Error> {
        let ctext = CString::new(text).unwrap();
        let ok = unsafe { TTF_SetTextString(self.raw, ctext.as_ptr(), ctext.count_bytes()) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "TTF_SetTextColor")]
    pub fn set_color<T>(&mut self, color: T) -> Result<(), Error>
    where
        T: Into<Color>,
    {
        let color: Color = color.into().into();
        let ok = unsafe { TTF_SetTextColor(self.raw, color.r, color.g, color.b, color.a) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }

    #[doc(alias = "TTF_SetTextWrapWidth")]
    pub fn set_wrap_width(&self, pixels: i32) -> Result<(), Error> {
        let ok = unsafe { TTF_SetTextWrapWidth(self.raw, pixels) };
        if ok {
            Ok(())
        } else {
            Err(get_error())
        }
    }
}
impl Drop for Text {
    fn drop(&mut self) {
        unsafe { TTF_DestroyText(self.raw) };
    }
}
