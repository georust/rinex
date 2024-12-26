use crate::{
    navigation::{
        EarthOrientation, Ephemeris, NavFrame, NavFrameType, NavKey, NavMessageType, SystemTime,
        NgModel, BdModel, KbModel,
    },
    prelude::ParsingError,
    prelude::SV,
};

/// ([NavKey], [NavFrame]) parsing attempt for a V4 frame.
/// In modern Navigation, all forms may exist.
pub fn parse(content: &str) -> Result<(NavKey, NavFrame), ParsingError> {
    let mut lines = content.lines();

    let line = match lines.next() {
        Some(l) => l,
        _ => return Err(ParsingError::EmptyEpoch),
    };

    let (_, rem) = line.split_at(2);
    let (class, rem) = rem.split_at(4);
    let (svnn, rem) = rem.split_at(4);

    // frmtype defines message to follow
    let frmtype = class.trim().parse::<NavFrameType>()?;
    let sv = svnn.trim().parse::<SV>()?;
    let msgtype = rem.trim().parse::<NavMessageType>()?;

    let ts = sv
        .constellation
        .timescale()
        .ok_or(ParsingError::NoTimescaleDefinition)?;

    // Parses navframe type dependent and epoch of publication
    let (epoch, fr) = match frmtype {
        NavFrameType::Ephemeris => {
            let (epoch, _, ephemeris) = Ephemeris::parse_v4(msgtype, lines, ts)?;
            (epoch, NavFrame::EPH(ephemeris))
        },
        NavFrameType::SystemTimeOffset => {
            let (epoch, system_time) = SystemTime::parse(lines, ts)?;
            (epoch, NavFrame::STO(system_time))
        },
        NavFrameType::EarthOrientation => {
            let (epoch, eop) = EarthOrientation::parse(lines, ts)?;
            (epoch, NavFrame::EOP(eop))
        },
        NavFrameType::IonosphereModel => {
            let (epoch, iono_model): (Epoch, IonMessage) = match msgtype {
                NavMsgType::IFNV => {
                    let (epoch, model) = NgModel::parse(lines, ts)?;
                    (epoch, IonosphereModel::NequickGModel(model))
                },
                NavMsgType::CNVX => match sv.constellation {
                    Constellation::BeiDou => {
                        let (epoch, model) = BdModel::parse(lines, ts)?;
                        (epoch, IonosphereModel::BdgimModel(model))
                    },
                    _ => {
                        let (epoch, model) = KbModel::parse(lines, ts)?;
                        (epoch, IonosphereModel::KlobucharModel(model))
                    },
                },
                _ => {
                    let (epoch, model) = KbModel::parse(lines, ts)?;
                    (epoch, IonosphereModel::KlobucharModel(model))
                },
            };
            (epoch, NavFrame::ION(msg))
        },
    };

    let key = NavKey {
        epoch,
        sv,
        msgtype,
        frmtype,
    };

    Ok((key, fr))
}
