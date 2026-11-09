
/// Internal stable replacement for !.
#[derive(Debug)]
pub enum Never {}

pub struct PanicOnDrop(pub &'static str);
impl Drop for PanicOnDrop {
    fn drop(&mut self) {
        panic!("{}", self.0);
    }
}


pub trait UnwrapNever<T> {
    fn unwrap_never(self) -> T;
}

impl<T> UnwrapNever<T> for Result<T, Never> {
    fn unwrap_never(self) -> T {
        match self {
            Ok(x) => x,
            Err(e) => match e {},
        }
    }
}
