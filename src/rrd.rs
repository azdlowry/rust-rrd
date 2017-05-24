use rrd_ffi::*;
use std::ffi::*;
use error::*;
use std;
use std::os::raw::c_char;
use std::collections::HashMap;

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

    pub fn update_single_f64(&self, timestamp: time_t, value: f64) -> Result<(), Error> {
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

    pub fn fetch(&self,
                 function: ConsolidationFunction,
                 start: time_t,
                 end: time_t,
                 step: u64)
                 -> Result<HashMap<String, &[f64]>, Error> {
        let mut num_of_data_sources: u64 = 0;
        let mut names_of_data_sources: *mut *mut ::std::os::raw::c_char = std::ptr::null_mut();
        let mut data: *mut rrd_value_t = std::ptr::null_mut();
        let mut start = start;
        let mut end = end;
        let mut step = step;

        let result = unsafe {
            rrd_fetch_r(self.filename.as_ptr(),
                        CString::new(function.to_string())?.as_ptr(),
                        &mut start,
                        &mut end,
                        &mut step,
                        &mut num_of_data_sources,
                        &mut names_of_data_sources,
                        &mut data)
        };
        // Also take ownership of names_of_data_sources and data

        if result != 0 {
            Err(Error::RrdError(unsafe { CString::from_raw(rrd_get_error()) }))
        } else {
            let name_slices = unsafe {
                std::slice::from_raw_parts(names_of_data_sources, num_of_data_sources as usize)
            };
            let names = name_slices
                .iter()
                .map(|name_slice| unsafe { CString::from_raw(*name_slice) }.into_string().unwrap());
            let data_slices =
                unsafe { std::slice::from_raw_parts(data, num_of_data_sources as usize) };
            let data = data_slices
                .iter()
                .map(|data_slice| unsafe { std::slice::from_raw_parts(data_slice, ((end - start) as u64 / step + 1) as usize) });
            Ok(names.zip(data).collect())
        }
    }
}

pub enum ConsolidationFunction {
    Average,
    Min,
    Max,
    Last,
    HwPredict,
    Seasonal,
    DevPredict,
    DevSeasonal,
    Failures,
    MhwPredict,
}

impl ConsolidationFunction {
    pub fn to_string(&self) -> &'static str {
        match *self {
            ConsolidationFunction::Average => "AVERAGE",
            ConsolidationFunction::Min => "MIN",
            ConsolidationFunction::Max => "MAX",
            ConsolidationFunction::Last => "LAST",
            ConsolidationFunction::HwPredict => "HWPREDICT",
            ConsolidationFunction::Seasonal => "SEASONAL",
            ConsolidationFunction::DevPredict => "DEVPREDICT",
            ConsolidationFunction::DevSeasonal => "DEVSEASONAL",
            ConsolidationFunction::Failures => "FAILURES",
            ConsolidationFunction::MhwPredict => "MHWPREDICT",
        }
    }
}

