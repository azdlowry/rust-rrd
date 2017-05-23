use std;
use std::ffi::CString;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        NullStr(err: std::ffi::NulError) {
            from()
        }
        RrdError(err: CString) {
            from()
        }
    }
}