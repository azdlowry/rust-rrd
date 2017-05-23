#![feature(untagged_unions)]

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[macro_use]
extern crate quick_error;

#[allow(unused)]
mod rrd_ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod error;
pub mod rrd;

#[cfg(test)]
mod tests {
    extern crate tempdir;
    extern crate time;
    use super::*;

    #[test]
    fn it_works() {
        let tmp_dir = tempdir::TempDir::new("it_works").unwrap();

        let mut db_name = tmp_dir.path().to_path_buf();
        db_name.push("test.rrd");

        let db = rrd::Database::create(db_name.to_str().unwrap().into(),
                                       None,
                                       Some(1000000),
                                       None,
                                       None,
                                       None,
                                       vec!["DS:speed:COUNTER:600:U:U",
                                            "RRA:AVERAGE:0.5:1:24",
                                            "RRA:AVERAGE:0.5:6:10"])
                .unwrap();

        for t in 1..1000 {
            db.update_single_f64(1000000 + (t * 1000), t as f64 * 1000.0)
                .unwrap();
        }

        // exit so we can see what's going on the database
        //std::process::exit(0);
    }
}
