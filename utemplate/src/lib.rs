pub use utemplate_macro::fmt;

pub trait TDisplay {
    fn tdisplay_to(self, dst: &mut String);
}

impl<'a> TDisplay for &'a str {
    fn tdisplay_to(self, dst: &mut String) {
        dst.push_str(self);
    }
}

fn integer_display_to<T: itoa::Integer>(value: T, dst: &mut String) {
    let mut buf = itoa::Buffer::new();
    dst.push_str(buf.format(value));
}

// TDisplay for Integer types
macro_rules! integer_tdisplay {
    ($($t:ty),*) => {
        $(
            impl TDisplay for $t {
                fn tdisplay_to(self, dst: &mut String) {
                    integer_display_to(self, dst);
                }
            }

            impl<'a> TDisplay for &'a $t {
                fn tdisplay_to(self, dst: &mut String) {
                    integer_display_to(*self, dst);
                }
            }
        )*
    };
}
// impl_display for all integer and float types:
integer_tdisplay!(u8);
integer_tdisplay!(u16);
integer_tdisplay!(u32);
integer_tdisplay!(u64);
integer_tdisplay!(u128);
integer_tdisplay!(usize);
integer_tdisplay!(i8);
integer_tdisplay!(i16);
integer_tdisplay!(i32);
integer_tdisplay!(i64);
integer_tdisplay!(i128);
integer_tdisplay!(isize);

fn float_display_to<T: ryu::Float>(value: T, dst: &mut String) {
    let mut buf = ryu::Buffer::new();
    dst.push_str(buf.format(value));
}

// TDisplay for Float types
macro_rules! float_tdisplay {
    ($($t:ty),*) => {
        $(
            impl TDisplay for $t {
                fn tdisplay_to(self, dst: &mut String) {
                    float_display_to(self, dst);
                }
            }

            impl<'a> TDisplay for &'a $t {
                fn tdisplay_to(self, dst: &mut String) {
                    float_display_to(*self, dst);
                }
            }
        )*
    };
}
float_tdisplay!(f32);
float_tdisplay!(f64);
