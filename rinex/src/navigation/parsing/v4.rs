use crate::{
    navigation::{Ephemeris, NavFrame, NavFrameType, NavKey, NavMessageType},
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
            // let (epoch, msg) = StoMessage::parse(lines, ts)?;
            // (epoch, NavFrame::Sto(msg_type, sv, msg))
            panic!("not yet");
        },
        NavFrameType::EarthOrientation => {
            // let (epoch, msg) = EopMessage::parse(lines, ts)?;
            // (epoch, NavFrame::Eop(msg_type, sv, msg))
            panic!("not yet");
        },
        NavFrameType::IonosphereModel => {
            // let (epoch, msg): (Epoch, IonMessage) = match msg_type {
            //     NavMsgType::IFNV => {
            //         let (epoch, model) = NgModel::parse(lines, ts)?;
            //         (epoch, IonMessage::NequickGModel(model))
            //     },
            //     NavMsgType::CNVX => match sv.constellation {
            //         Constellation::BeiDou => {
            //             let (epoch, model) = BdModel::parse(lines, ts)?;
            //             (epoch, IonMessage::BdgimModel(model))
            //         },
            //         _ => {
            //             let (epoch, model) = KbModel::parse(lines, ts)?;
            //             (epoch, IonMessage::KlobucharModel(model))
            //         },
            //     },
            //     _ => {
            //         let (epoch, model) = KbModel::parse(lines, ts)?;
            //         (epoch, IonMessage::KlobucharModel(model))
            //     },
            // };
            // (epoch, NavFrame::Ion(msg_type, sv, msg))
            panic!("not yet");
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
