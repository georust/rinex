// #[cfg(test)]
// mod test {
//     use crate::prelude::*;
//     use crate::tests::toolkit::test_meteo_rinex;
//     use crate::{erratic_time_frame, evenly_spaced_time_frame, tests::toolkit::TestTimeFrame};
//     use itertools::Itertools;
//     use std::str::FromStr;
//     #[test]
//     fn v2_abvi0010_15m() {
//         let test_resource =
//             env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/MET/V2/abvi0010.15m";
//         let rinex = Rinex::from_file::<5>(&test_resource);
//         assert!(rinex.is_ok());
//         let rinex = rinex.unwrap();
//         test_meteo_rinex(
//             &rinex,
//             "2.11",
//             "PR, TD, HR, WS, WD, RI, HI",
//             erratic_time_frame!(
//                 "
//                 2015-01-01T00:00:00 UTC,
//                 2015-01-01T00:01:00 UTC,
//                 2015-01-01T00:02:00 UTC,
//                 2015-01-01T00:03:00 UTC,
//                 2015-01-01T00:04:00 UTC,
//                 2015-01-01T00:05:00 UTC,
//                 2015-01-01T00:06:00 UTC,
//                 2015-01-01T00:07:00 UTC,
//                 2015-01-01T00:08:00 UTC,
//                 2015-01-01T00:09:00 UTC,
//                 2015-01-01T09:00:00 UTC,
//                 2015-01-01T09:01:00 UTC,
//                 2015-01-01T09:02:00 UTC,
//                 2015-01-01T09:03:00 UTC,
//                 2015-01-01T09:04:00 UTC,
//                 2015-01-01T19:25:00 UTC,
//                 2015-01-01T19:26:00 UTC,
//                 2015-01-01T19:27:00 UTC,
//                 2015-01-01T19:28:00 UTC,
//                 2015-01-01T19:29:00 UTC,
//                 2015-01-01T19:30:00 UTC,
//                 2015-01-01T19:31:00 UTC,
//                 2015-01-01T19:32:00 UTC,
//                 2015-01-01T19:33:00 UTC,
//                 2015-01-01T19:34:00 UTC,
//                 2015-01-01T19:35:00 UTC,
//                 2015-01-01T19:36:00 UTC,
//                 2015-01-01T19:37:00 UTC,
//                 2015-01-01T19:38:00 UTC,
//                 2015-01-01T19:39:00 UTC,
//                 2015-01-01T19:40:00 UTC,
//                 2015-01-01T19:41:00 UTC,
//                 2015-01-01T19:42:00 UTC,
//                 2015-01-01T19:43:00 UTC,
//                 2015-01-01T19:44:00 UTC,
//                 2015-01-01T19:45:00 UTC,
//                 2015-01-01T19:46:00 UTC,
//                 2015-01-01T19:47:00 UTC,
//                 2015-01-01T19:48:00 UTC,
//                 2015-01-01T19:49:00 UTC,
//                 2015-01-01T19:50:00 UTC,
//                 2015-01-01T19:51:00 UTC,
//                 2015-01-01T19:52:00 UTC,
//                 2015-01-01T19:53:00 UTC,
//                 2015-01-01T19:54:00 UTC,
//                 2015-01-01T22:55:00 UTC,
//                 2015-01-01T22:56:00 UTC,
//                 2015-01-01T22:57:00 UTC,
//                 2015-01-01T22:58:00 UTC,
//                 2015-01-01T22:59:00 UTC,
//                 2015-01-01T23:01:00 UTC,
//                 2015-01-01T23:01:00 UTC,
//                 2015-01-01T23:02:00 UTC,
//                 2015-01-01T23:09:00 UTC,
//                 2015-01-01T23:10:00 UTC,
//                 2015-01-01T23:11:00 UTC,
//                 2015-01-01T23:12:00 UTC,
//                 2015-01-01T23:13:00 UTC,
//                 2015-01-01T23:14:00 UTC,
//                 2015-01-01T23:15:00 UTC,
//                 2015-01-01T23:16:00 UTC,
//                 2015-01-01T23:17:00 UTC,
//                 2015-01-01T23:18:00 UTC,
//                 2015-01-01T23:19:00 UTC,
//                 2015-01-01T23:20:00 UTC,
//                 2015-01-01T23:21:00 UTC,
//                 2015-01-01T23:52:00 UTC,
//                 2015-01-01T23:53:00 UTC,
//                 2015-01-01T23:54:00 UTC,
//                 2015-01-01T23:55:00 UTC,
//                 2015-01-01T23:56:00 UTC,
//                 2015-01-01T23:57:00 UTC,
//                 2015-01-01T23:58:00 UTC,
//                 2015-01-01T23:59:00 UTC
//             "
//             ),
//         );
// =======
// // #[cfg(test)]
// // mod test {
// //     use crate::prelude::*;
// //     use crate::tests::toolkit::test_meteo_rinex;
// //     use crate::{erratic_time_frame, evenly_spaced_time_frame, tests::toolkit::TestTimeFrame};
// //     use itertools::Itertools;
// //     use std::str::FromStr;
// //     #[test]
// //     fn v2_abvi0010_15m() {
// //         let test_resource =
// //             env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/MET/V2/abvi0010.15m";
// //         let rinex = Rinex::from_file(&test_resource);
// //         assert!(rinex.is_ok());
// //         let rinex = rinex.unwrap();
// //         test_meteo_rinex(
// //             &rinex,
// //             "2.11",
// //             "PR, TD, HR, WS, WD, RI, HI",
// //             erratic_time_frame!(
// //                 "
// //                 2015-01-01T00:00:00 UTC,
// //                 2015-01-01T00:01:00 UTC,
// //                 2015-01-01T00:02:00 UTC,
// //                 2015-01-01T00:03:00 UTC,
// //                 2015-01-01T00:04:00 UTC,
// //                 2015-01-01T00:05:00 UTC,
// //                 2015-01-01T00:06:00 UTC,
// //                 2015-01-01T00:07:00 UTC,
// //                 2015-01-01T00:08:00 UTC,
// //                 2015-01-01T00:09:00 UTC,
// //                 2015-01-01T09:00:00 UTC,
// //                 2015-01-01T09:01:00 UTC,
// //                 2015-01-01T09:02:00 UTC,
// //                 2015-01-01T09:03:00 UTC,
// //                 2015-01-01T09:04:00 UTC,
// //                 2015-01-01T19:25:00 UTC,
// //                 2015-01-01T19:26:00 UTC,
// //                 2015-01-01T19:27:00 UTC,
// //                 2015-01-01T19:28:00 UTC,
// //                 2015-01-01T19:29:00 UTC,
// //                 2015-01-01T19:30:00 UTC,
// //                 2015-01-01T19:31:00 UTC,
// //                 2015-01-01T19:32:00 UTC,
// //                 2015-01-01T19:33:00 UTC,
// //                 2015-01-01T19:34:00 UTC,
// //                 2015-01-01T19:35:00 UTC,
// //                 2015-01-01T19:36:00 UTC,
// //                 2015-01-01T19:37:00 UTC,
// //                 2015-01-01T19:38:00 UTC,
// //                 2015-01-01T19:39:00 UTC,
// //                 2015-01-01T19:40:00 UTC,
// //                 2015-01-01T19:41:00 UTC,
// //                 2015-01-01T19:42:00 UTC,
// //                 2015-01-01T19:43:00 UTC,
// //                 2015-01-01T19:44:00 UTC,
// //                 2015-01-01T19:45:00 UTC,
// //                 2015-01-01T19:46:00 UTC,
// //                 2015-01-01T19:47:00 UTC,
// //                 2015-01-01T19:48:00 UTC,
// //                 2015-01-01T19:49:00 UTC,
// //                 2015-01-01T19:50:00 UTC,
// //                 2015-01-01T19:51:00 UTC,
// //                 2015-01-01T19:52:00 UTC,
// //                 2015-01-01T19:53:00 UTC,
// //                 2015-01-01T19:54:00 UTC,
// //                 2015-01-01T22:55:00 UTC,
// //                 2015-01-01T22:56:00 UTC,
// //                 2015-01-01T22:57:00 UTC,
// //                 2015-01-01T22:58:00 UTC,
// //                 2015-01-01T22:59:00 UTC,
// //                 2015-01-01T23:01:00 UTC,
// //                 2015-01-01T23:01:00 UTC,
// //                 2015-01-01T23:02:00 UTC,
// //                 2015-01-01T23:09:00 UTC,
// //                 2015-01-01T23:10:00 UTC,
// //                 2015-01-01T23:11:00 UTC,
// //                 2015-01-01T23:12:00 UTC,
// //                 2015-01-01T23:13:00 UTC,
// //                 2015-01-01T23:14:00 UTC,
// //                 2015-01-01T23:15:00 UTC,
// //                 2015-01-01T23:16:00 UTC,
// //                 2015-01-01T23:17:00 UTC,
// //                 2015-01-01T23:18:00 UTC,
// //                 2015-01-01T23:19:00 UTC,
// //                 2015-01-01T23:20:00 UTC,
// //                 2015-01-01T23:21:00 UTC,
// //                 2015-01-01T23:52:00 UTC,
// //                 2015-01-01T23:53:00 UTC,
// //                 2015-01-01T23:54:00 UTC,
// //                 2015-01-01T23:55:00 UTC,
// //                 2015-01-01T23:56:00 UTC,
// //                 2015-01-01T23:57:00 UTC,
// //                 2015-01-01T23:58:00 UTC,
// //                 2015-01-01T23:59:00 UTC
// //             "
// //             ),
// //         );
//
// //         let labels = [
// //             "pressure",
// //             "temp",
// //             "moisture",
// //             "wind-speed",
// //             "wind-direction",
// //             "rain inc.",
// //         ];
// //         let expected = vec![
// //             (
// //                 0,
// //                 Epoch::from_str("2015-01-01T00:00:00 UTC"),
// //                 vec![1018.6, 25.6, 78.9, 3.1, 10.0, 0.0],
// //             ),
// //             (
// //                 1,
// //                 Epoch::from_str("2015-01-01T00:01:00 UTC"),
// //                 vec![1018.7, 25.6, 79.4, 2.1, 7.0, 0.0],
// //             ),
// //             (
// //                 2,
// //                 Epoch::from_str("2015-01-01T00:02:00 UTC"),
// //                 vec![1018.6, 25.5, 79.6, 2.0, 3.0, 0.0],
// //             ),
// //             (
// //                 3,
// //                 Epoch::from_str("2015-01-01T00:03:00 UTC"),
// //                 vec![1018.7, 25.5, 80.0, 1.9, 8.0, 0.0],
// //             ),
// //             (
// //                 4,
// //                 Epoch::from_str("2015-01-01T00:04:00 UTC"),
// //                 vec![1018.7, 25.4, 80.4, 3.9, 11.0, 0.0],
// //             ),
// //             (
// //                 5,
// //                 Epoch::from_str("2015-01-01T00:05:00 UTC"),
// //                 vec![1018.7, 25.4, 80.5, 1.6, 20.0, 0.0],
// //             ),
// //             (
// //                 17,
// //                 Epoch::from_str("2015-01-01T19:27:00 UTC"),
// //                 vec![1018.4, 28.5, 65.9, 2.6, 351.0, 0.0],
// //             ),
// //             (
// //                 71,
// //                 Epoch::from_str("2015-01-01T23:57:00 UTC"),
// //                 vec![1019.8, 25.8, 73.8, 1.7, 338.0, 0.0],
// //             ),
// //             (
// //                 72,
// //                 Epoch::from_str("2015-01-01T23:58:00 UTC"),
// //                 vec![1019.8, 25.8, 73.8, 3.6, 341.0, 0.0],
// //             ),
// //             (
// //                 73,
// //                 Epoch::from_str("2015-01-01T23:59:00 UTC"),
// //                 vec![1019.8, 25.8, 72.8, 4.8, 4.0, 0.0],
// //             ),
// //         ];
//
// //         let epochs = rinex.epoch().collect::<Vec<Epoch>>();
//
// //         let record_values: Vec<Vec<(Epoch, f64)>> = vec![
// //             rinex.pressure().collect(),
// //             rinex.temperature().collect(),
// //             rinex.moisture().collect(),
// //             rinex.wind_speed().collect(),
// //             rinex.wind_direction().collect(),
// //             rinex.rain_increment().collect(),
// //         ];
//
// //         for expected_values in expected {
// //             let (index, epoch, expected_values) = expected_values;
// //             let epoch = epoch.unwrap();
//
// //             let content = epochs.get(index as usize);
// //             assert!(content.is_some(), "missing epoch {}", epoch);
//
// //             //let content = content.unwrap();
// //             for (field_index, expected_value) in expected_values.iter().enumerate() {
// //                 let label = labels[field_index];
// //                 let value = record_values[field_index].get(index as usize);
// //                 assert!(
// //                     value.is_some(),
// //                     "{} : missing \"{}\" measurement",
// //                     epoch,
// //                     label
// //                 );
// //                 let value = value.unwrap().1;
// //                 assert!(
// //                     value == *expected_value,
// //                     "{}({}): found wrong value \"{}\" instead of \"{}\"",
// //                     epoch,
// //                     label,
// //                     value,
// //                     expected_value
// //                 );
// //             }
// //         }
//
// //         let meteo_iters = vec![
// //             ("temperature", rinex.temperature(), 74),
// //             ("pressure", rinex.pressure(), 74),
// //             ("moisture", rinex.moisture(), 74),
// //             ("zenith (dry)", rinex.zenith_dry_delay(), 0),
// //             ("zenith (wet)", rinex.zenith_wet_delay(), 0),
// //             ("zenith (tot)", rinex.zenith_delay(), 0),
// //         ];
//
// <<<<<<< HEAD
//         for (test, iter, expected) in meteo_iters {
//             assert!(
//                 iter.count() == expected,
//                 "\"{}\": parsed wrong amount of data",
//                 test
//             );
//         }
//         assert_eq!(
//             rinex.accumulated_rain(),
//             0.0,
//             "Error: it did not rain on that day"
//         );
//         assert!(!rinex.hail_detected(), "Error: it did not hail on that day");
//     }
//     #[test]
//     fn v4_example1() {
//         let test_resource =
//             env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/MET/V4/example1.txt";
//         let rinex = Rinex::from_file::<5>(&test_resource);
//         assert!(rinex.is_ok());
//         let rinex = rinex.unwrap();
//         test_meteo_rinex(
//             &rinex,
//             "4.00",
//             "PR, TD, HR",
//             evenly_spaced_time_frame!("2021-01-07T00:00:00 UTC", "2021-01-07T00:02:00 UTC", "30 s"),
//         );
// =======
// //         for (test, iter, expected) in meteo_iters {
// //             assert!(
// //                 iter.count() == expected,
// //                 "\"{}\": parsed wrong amount of data",
// //                 test
// //             );
// //         }
// //         assert_eq!(
// //             rinex.accumulated_rain(),
// //             0.0,
// //             "Error: it did not rain on that day"
// //         );
// //         assert!(!rinex.hail_detected(), "Error: it did not hail on that day");
// //     }
// //     #[test]
// //     fn v4_example1() {
// //         let test_resource =
// //             env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/MET/V4/example1.txt";
// //         let rinex = Rinex::from_file(&test_resource);
// //         assert!(rinex.is_ok());
// //         let rinex = rinex.unwrap();
// //         test_meteo_rinex(
// //             &rinex,
// //             "4.00",
// //             "PR, TD, HR",
// //             evenly_spaced_time_frame!("2021-01-07T00:00:00 UTC", "2021-01-07T00:02:00 UTC", "30 s"),
// //         );
//
// //         let record = rinex.record.as_meteo();
// //         assert!(record.is_some());
// //         let record = record.unwrap();
// //         assert_eq!(record.len(), 5);
//
// //         // test epoch content
// //         for (_, obs) in record.iter() {
// //             for (obs, data) in obs.iter() {
// //                 if *obs == Observable::Pressure {
// //                     assert_eq!(*data, 993.3);
// //                 } else if *obs == Observable::HumidityRate {
// //                     assert_eq!(*data, 90.0);
// //                 }
// //             }
// //         }
// //         let epoch = Epoch::from_gregorian_utc(2021, 1, 7, 00, 00, 00, 00);
// //         let e = record.get(&epoch).unwrap();
// //         for (obs, data) in e.iter() {
// //             if *obs == Observable::Temperature {
// //                 assert_eq!(*data, 23.0);
// //             }
// //         }
// //         let epoch = Epoch::from_gregorian_utc(2021, 1, 7, 0, 0, 30, 0);
// //         let e = record.get(&epoch).unwrap();
// //         for (obs, data) in e.iter() {
// //             if *obs == Observable::Temperature {
// //                 assert_eq!(*data, 23.0);
// //             }
// //         }
// //         let epoch = Epoch::from_gregorian_utc(2021, 1, 7, 0, 1, 0, 00);
// //         let e = record.get(&epoch).unwrap();
// //         for (obs, data) in e.iter() {
// //             if *obs == Observable::Temperature {
// //                 assert_eq!(*data, 23.1);
// //             }
// //         }
// //         let epoch = Epoch::from_gregorian_utc(2021, 1, 7, 0, 1, 30, 0);
// //         let e = record.get(&epoch).unwrap();
// //         for (obs, data) in e.iter() {
// //             if *obs == Observable::Temperature {
// //                 assert_eq!(*data, 23.1);
// //             }
// //         }
// //         let epoch = Epoch::from_gregorian_utc(2021, 1, 7, 0, 2, 0, 00);
// //         let e = record.get(&epoch).unwrap();
// //         for (obs, data) in e.iter() {
// //             if *obs == Observable::Temperature {
// //                 assert_eq!(*data, 23.1);
// //             }
// //         }
// //     }
// // }
