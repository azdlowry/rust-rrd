#![feature(untagged_unions,test)]

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
    extern crate test;
    use super::*;
    use self::test::Bencher;

    #[test]
    fn can_roundtrip_some_data() {
        let tmp_dir = tempdir::TempDir::new("can_roundtrip_some_data").unwrap();

        let mut db_name = tmp_dir.path().to_path_buf();
        db_name.push("test.rrd");

        let db = rrd::Database::create(db_name.to_str().unwrap().into(),
                                       None,
                                       Some(1000000),
                                       None,
                                       None,
                                       None,
                                       vec!["DS:speed:GAUGE:6000:U:U",
                                            "RRA:AVERAGE:0.5:1:240",
                                            "RRA:AVERAGE:0.5:6:100"])
                .unwrap();

        for t in 1..10 {
            db.update_single_f64(1000000 + (t * 1000), 4337.0)
                .unwrap();
        }

        let data = db.fetch(rrd::ConsolidationFunction::Average,
                            1000000 + 1000,
                            1000000 + (7 * 1000),
                            1000)
            .unwrap();

        println!("Data: {:?}", data);

        assert_eq!(4337.0, data["speed"][1]);

        // exit so we can see what's going on the database
        //std::process::exit(0);
    }

    #[test]
    fn can_open_an_existing_database() {
        let tmp_dir = tempdir::TempDir::new("can_open_an_existing_database").unwrap();

        let mut db_name = tmp_dir.path().to_path_buf();
        db_name.push("test.rrd");

        {
            rrd::Database::create(db_name.to_str().unwrap().into(),
                                  None,
                                  Some(1000000),
                                  None,
                                  None,
                                  None,
                                  vec!["DS:speed:GAUGE:6000:U:U",
                                       "RRA:AVERAGE:0.5:1:240",
                                       "RRA:AVERAGE:0.5:6:100"])
                    .unwrap();
        }

        let db = rrd::Database::open(db_name.to_str().unwrap().into()).unwrap();

        for t in 1..10 {
            db.update_single_f64(1000000 + (t * 1000), 4337.0)
                .unwrap();
        }

        let data = db.fetch(rrd::ConsolidationFunction::Average,
                            1000000 + 1000,
                            1000000 + (7 * 1000),
                            1000)
            .unwrap();

        println!("Data: {:?}", data);

        assert_eq!(4337.0, data["speed"][1]);

        // exit so we can see what's going on the database
        //std::process::exit(0);
    }

    #[test]
    fn can_update_multiple_points_in_one_call() {
        let tmp_dir = tempdir::TempDir::new("can_open_an_existing_database").unwrap();

        let mut db_name = tmp_dir.path().to_path_buf();
        db_name.push("test.rrd");

        let db = rrd::Database::create(db_name.to_str().unwrap().into(),
                                       None,
                                       Some(1000000),
                                       None,
                                       None,
                                       None,
                                       vec!["DS:speed:GAUGE:6000:U:U",
                                            "RRA:AVERAGE:0.5:1:240",
                                            "RRA:AVERAGE:0.5:6:100"])
                .unwrap();
        rrd::Database::open(db_name.to_str().unwrap().into()).unwrap();

        db.update_f64((1..10).map(|t| (1000000 + (t * 1000), 4337.0)).collect())
            .unwrap();

        let data = db.fetch(rrd::ConsolidationFunction::Average,
                            1000000 + 1000,
                            1000000 + (7 * 1000),
                            1000)
            .unwrap();

        println!("Data: {:?}", data);

        assert_eq!(4337.0, data["speed"][1]);

        // exit so we can see what's going on the database
        //std::process::exit(0);
    }

    #[bench]
    fn benchmark_single_updates(b: &mut Bencher) {
        let tmp_dir = tempdir::TempDir::new("can_open_an_existing_database").unwrap();

        let mut db_name = tmp_dir.path().to_path_buf();
        db_name.push("test.rrd");

        let db = rrd::Database::create(db_name.to_str().unwrap().into(),
                                       None,
                                       Some(1000000),
                                       None,
                                       None,
                                       None,
                                       vec!["DS:speed:GAUGE:6000:U:U",
                                            "RRA:AVERAGE:0.5:1:240",
                                            "RRA:AVERAGE:0.5:6:100"])
                .unwrap();

        let mut last_val = 1000000;
        let count = 100;
        b.iter(|| {
                   for t in 1..count {
                       db.update_single_f64(last_val + (t * 1000), 4337.0)
                           .unwrap();
                   }

                   last_val += count * 1000;
               });
    }

    #[bench]
    fn benchmark_multi_updates(b: &mut Bencher) {
        let tmp_dir = tempdir::TempDir::new("can_open_an_existing_database").unwrap();

        let mut db_name = tmp_dir.path().to_path_buf();
        db_name.push("test.rrd");

        let db = rrd::Database::create(db_name.to_str().unwrap().into(),
                                       None,
                                       Some(1000000),
                                       None,
                                       None,
                                       None,
                                       vec!["DS:speed:GAUGE:6000:U:U",
                                            "RRA:AVERAGE:0.5:1:240",
                                            "RRA:AVERAGE:0.5:6:100"])
                .unwrap();

        let mut last_val = 1000000;
        let count = 100;
        b.iter(|| {
                   db.update_f64((1..count)
                                     .map(|t| (last_val + (t * 1000), 4337.0))
                                     .collect())
                       .unwrap();
                   last_val += count * 1000;
               });
    }
}
