use rrd_ffi::*;
use std::ffi::*;
use error::*;
use std;
use std::os::raw::c_char;

pub struct Database {
    filename: CString,
}

impl Database {
    pub fn create(filename: String,
                  pdp_step: Option<u64>,
                  last_up: Option<time_t>,
                  no_overwrite: Option<bool>,
                  sources: Option<Vec<String>>,
                  template: Option<String>,
                  argv: Vec<&str>)
                  -> Result<Database, Error> {
        let filename = CString::new(&*filename)?;
        let argv = argv.into_iter() // Need to keep argv in scope so that the strings are not dropped
            .map(|arg| CString::new(arg).unwrap())
            .collect::<Vec<CString>>();
        let mut args = argv.iter()
            .map(|arg| arg.as_ptr())
            .collect::<Vec<*const c_char>>();
        args.push(std::ptr::null());

        let result = unsafe {
            rrd_create_r2(filename.as_ptr(),
                          pdp_step.unwrap_or(0),
                          last_up.unwrap_or(0),
                          if no_overwrite.unwrap_or(false) { 1 } else { 0 },
                          if let Some(s) = sources {
                              s.into_iter()
                                  .map(move |c| CString::new(c).unwrap().as_ptr())
                                  .collect::<Vec<*const i8>>()
                                  .as_mut_ptr()
                          } else {
                              std::ptr::null_mut()
                          },
                          if let Some(t) = template {
                              CString::new(&*t)?.as_ptr()
                          } else {
                              std::ptr::null_mut()
                          },
                          argv.len() as i32,
                          args.as_mut_ptr())
        };

        if result != 0 {
            Err(Error::RrdError(unsafe { CString::from_raw(rrd_get_error()) }))
        } else {
            Ok(Database { filename: filename })
        }
    }

    pub fn update_single_f64(&self,
                             timestamp: time_t,
                             value: f64)
                             -> Result<(), Error> {
        let argv = vec![format!("{}:{}", timestamp, value)];

        let argv = argv.into_iter() // Need to keep argv in scope so that the strings are not dropped
            .map(|arg| CString::new(arg).unwrap())
            .collect::<Vec<CString>>();
        let mut args = argv.iter()
            .map(|arg| arg.as_ptr())
            .collect::<Vec<*const c_char>>();
        args.push(std::ptr::null());

        let result = unsafe {
            rrd_updatex_r(self.filename.as_ptr(),
                            std::ptr::null_mut(),
                            0,
                            argv.len() as i32,
                            args.as_mut_ptr())
        };

        if result != 0 {
            Err(Error::RrdError(unsafe { CString::from_raw(rrd_get_error()) }))
        } else {
            Ok(())
        }
    }
}
