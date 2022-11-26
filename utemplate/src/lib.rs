pub use utemplate_macro::fmt;
use lexical_core::ToLexical;

pub trait TDisplay {
    fn tdisplay_to(self, dst: &mut String);
}

impl<'a> TDisplay for &'a str {
    fn tdisplay_to(self, dst: &mut String) {
        dst.push_str(self);
    }
}

unsafe fn lexical_display_to<T: ToLexical>(value: T, dst: &mut String) {
    // Safety: for our trusted instances of ToLexical, it is guaranteed to return UTF-8.
    let dst = dst.as_mut_vec();
    let orig_len = dst.len();
    dst.resize(orig_len + T::FORMATTED_SIZE_DECIMAL, 0);
    // Safety: we have (exactly) T::FORMATTED_SIZE_DECIMAL space in the buffer.
    let written_len = value.to_lexical_unchecked(&mut dst[orig_len..]).len();
    // Safety: we have written exactly `written_len` bytes past the original length.
    dst.set_len(orig_len + written_len);
}

// TDisplay for ToLexical types
macro_rules! impl_tdisplay {
    ($($t:ty),*) => {
        $(
            impl TDisplay for $t {
                fn tdisplay_to(self, dst: &mut String) {
                    unsafe {
                        // Safety: we trust the implementation of ToLexical on the types
                        // below.
                        lexical_display_to(self, dst);
                    }
                }
            }

            impl<'a> TDisplay for &'a $t {
                fn tdisplay_to(self, dst: &mut String) {
                    TDisplay::tdisplay_to(*self, dst);
                }
            }
        )*
    };
}
// impl_display for all integer and float types:
impl_tdisplay!(u8);
impl_tdisplay!(u16);
impl_tdisplay!(u32);
impl_tdisplay!(u64);
impl_tdisplay!(u128);
impl_tdisplay!(usize);
impl_tdisplay!(i8);
impl_tdisplay!(i16);
impl_tdisplay!(i32);
impl_tdisplay!(i64);
impl_tdisplay!(i128);
impl_tdisplay!(isize);
impl_tdisplay!(f32);
impl_tdisplay!(f64);
