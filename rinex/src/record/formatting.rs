use crate::record::Record;

impl Record {
    // /// Streams into given file writer
    // pub fn to_file<W: Write>(
    //     &self,
    //     header: &header::Header,
    //     writer: &mut BufferedWriter<W>,
    // ) -> Result<(), FormattingError> {
    //     let major = header.version.major;
    //     let constell = header.constellation;

    //     match &header.rinex_type {
    //         Type::MeteoData => {
    //             let record = self.as_meteo().unwrap();
    //             for (epoch, data) in record.iter() {
    //                 if let Ok(epoch) = meteo::record::fmt_epoch(epoch, data, header) {
    //                     let _ = write!(writer, "{}", epoch);
    //                 }
    //             }
    //         },
    //         Type::ObservationData => {
    //             let record = self.as_obs().unwrap();
    //             let obs_fields = &header.obs.as_ref().unwrap();
    //         },
    //         Type::NavigationData => {
    //             let record = self.as_nav().unwrap();
    //             for (epoch, frames) in record.iter() {
    //                 if let Ok(epoch) = navigation::record::fmt_epoch(epoch, frames, header) {
    //                     let _ = write!(writer, "{}", epoch);
    //                 }
    //             }
    //         },
    //         Type::ClockData => {
    //             if let Some(rec) = self.as_clock() {
    //                 for (epoch, keys) in rec {
    //                     for (key, prof) in keys {
    //                         let _ =
    //                             write!(writer, "{}", clock::record::fmt_epoch(epoch, key, prof));
    //                     }
    //                 }
    //             }
    //         },
    //         Type::IonosphereMaps => {
    //             if let Some(_r) = self.as_ionex() {
    //                 //for (index, (epoch, (_map, _, _))) in r.iter().enumerate() {
    //                 //    let _ = write!(writer, "{:6}                                                      START OF TEC MAP", index);
    //                 //    let _ = write!(
    //                 //        writer,
    //                 //        "{}                        EPOCH OF CURRENT MAP",
    //                 //        epoch::format(*epoch, None, Type::IonosphereMaps, 1)
    //                 //    );
    //                 //    let _ = write!(writer, "{:6}                                                      END OF TEC MAP", index);
    //                 //}
    //                 // /*
    //                 //  * not efficient browsing, but matches provided examples and common formatting.
    //                 //  * RMS and Height maps are passed after TEC maps.
    //                 //  */
    //                 //for (index, (epoch, (_, _map, _))) in r.iter().enumerate() {
    //                 //    let _ = write!(writer, "{:6}                                                      START OF RMS MAP", index);
    //                 //    let _ = write!(
    //                 //        writer,
    //                 //        "{}                        EPOCH OF CURRENT MAP",
    //                 //        epoch::format(*epoch, None, Type::IonosphereMaps, 1)
    //                 //    );
    //                 //    let _ = write!(writer, "{:6}                                                      END OF RMS MAP", index);
    //                 //}
    //                 //for (index, (epoch, (_, _, _map))) in r.iter().enumerate() {
    //                 //    let _ = write!(writer, "{:6}                                                      START OF HEIGHT MAP", index);
    //                 //    let _ = write!(
    //                 //        writer,
    //                 //        "{}                        EPOCH OF CURRENT MAP",
    //                 //        epoch::format(*epoch, None, Type::IonosphereMaps, 1)
    //                 //    );
    //                 //    let _ = write!(writer, "{:6}                                                      END OF HEIGHT MAP", index);
    //                 //}
    //             }
    //         },
    //         _ => panic!("record type not supported yet"),
    //     }
    //     Ok(())
    // }
}
