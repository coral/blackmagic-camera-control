use fixed::types::I5F11;
use num::{FromPrimitive, ToPrimitive};
use std::convert::TryInto;

pub type BMFixed = I5F11;
#[derive(Debug)]
pub enum Data {
    Void,
    Bool(bool),
    SignedByte(Vec<i8>),
    Signed16(Vec<i16>),
    Signed32(Vec<i32>),
    Signed64(Vec<i64>),
    String(String),
    FixedPoint(Vec<BMFixed>),
    Undefined,
}

impl Data {
    pub fn get_type(&self) -> u8 {
        match self {
            Data::Void => 0,
            Data::Bool(_) => 0,
            Data::SignedByte(_) => 1,
            Data::Signed16(_) => 2,
            Data::Signed32(_) => 3,
            Data::Signed64(_) => 4,
            Data::String(_) => 5,
            Data::FixedPoint(_) => 6,
            Data::Undefined => 255,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Data::Void => vec![],
            Data::Bool(v) => {
                if *v {
                    vec![1]
                } else {
                    vec![0]
                }
            }
            Data::SignedByte(v) => v.iter().map(|chunk| *chunk as u8).collect(),
            Data::Signed16(v) => {
                let mut ret = Vec::new();
                for i in v.iter() {
                    for b in i.to_le_bytes() {
                        ret.push(b);
                    }
                }
                ret
            }
            Data::Signed32(v) => {
                let mut ret = Vec::new();
                for i in v.iter() {
                    for b in i.to_le_bytes() {
                        ret.push(b);
                    }
                }
                ret
            }
            Data::Signed64(v) => {
                let mut ret = Vec::new();
                for i in v.iter() {
                    for b in i.to_le_bytes() {
                        ret.push(b);
                    }
                }
                ret
            }
            Data::String(v) => v.as_bytes().to_vec(),
            Data::FixedPoint(v) => {
                let mut ret = Vec::new();
                for i in v.iter() {
                    for b in i.to_le_bytes() {
                        ret.push(b);
                    }
                }
                ret
            }
            Data::Undefined => vec![],
        }
    }

    pub fn decode(length: u8, cmd_type: u8, data: Vec<u8>) -> Data {
        let datalen = length as usize - 4;
        let len: usize = match cmd_type {
            0 => 1,
            1 => 1,
            2 => 2,
            3 => 4,
            4 => 8,
            5 => datalen,
            128 => 2,
            _ => 0,
        };

        match cmd_type {
            0 => {
                if length == 4 {
                    Data::Void
                } else {
                    if data[0] > 0 {
                        Data::Bool(true)
                    } else {
                        Data::Bool(false)
                    }
                }
            }
            1 => Data::SignedByte(data[..datalen].iter().map(|chunk| *chunk as i8).collect()),
            2 => Data::Signed16(
                data[..datalen]
                    .chunks_exact(len)
                    .map(|chunk| i16::from_le_bytes(chunk.try_into().unwrap()))
                    .collect(),
            ),
            3 => Data::Signed32(
                data[..datalen]
                    .chunks_exact(len)
                    .map(|chunk| i32::from_le_bytes(chunk.try_into().unwrap()))
                    .collect(),
            ),
            4 => Data::Signed64(
                data[..datalen]
                    .chunks_exact(len)
                    .map(|chunk| i64::from_le_bytes(chunk.try_into().unwrap()))
                    .collect(),
            ),
            5 => Data::String(String::from_utf8(data).unwrap()),
            128 => Data::FixedPoint(
                data[..datalen]
                    .chunks_exact(len)
                    .map(|chunk| BMFixed::from_le_bytes(chunk.try_into().unwrap()))
                    .collect(),
            ),
            _ => Data::Undefined,
        }
    }
}

#[derive(Debug, PartialEq, Primitive)]
pub enum Operation {
    AssignValue = 0,
    OffsetValue = 1,
    Unknown = 2,
}

#[derive(Debug, PartialEq)]
pub enum Category {
    Lens(Lens),
    Video(Video),
    Audio(Audio),
    Output(Output),
    Display(Display),
    Tally(Tally),
    Reference(Reference),
    Configuration(Configuration),
    ColorCorrection(ColorCorrection),
    Internal,
    Media(Media),
    PTZ(PTZ),
    Metadata(Metadata),
    Unknown,
}

impl Category {
    pub fn lookup(category: u8, parameter: u8) -> Category {
        match category {
            0 => Category::Lens(match Lens::from_u8(parameter) {
                Some(v) => v,
                None => Lens::Unknown,
            }),
            1 => Category::Video(match Video::from_u8(parameter) {
                Some(v) => v,
                None => Video::Unknown,
            }),
            2 => Category::Audio(match Audio::from_u8(parameter) {
                Some(v) => v,
                None => Audio::Unknown,
            }),
            3 => Category::Output(match Output::from_u8(parameter) {
                Some(v) => v,
                None => Output::Unknown,
            }),
            4 => Category::Display(match Display::from_u8(parameter) {
                Some(v) => v,
                None => Display::Unknown,
            }),
            5 => Category::Tally(match Tally::from_u8(parameter) {
                Some(v) => v,
                None => Tally::Unknown,
            }),
            6 => Category::Reference(match Reference::from_u8(parameter) {
                Some(v) => v,
                None => Reference::Unknown,
            }),
            7 => Category::Configuration(match Configuration::from_u8(parameter) {
                Some(v) => v,
                None => Configuration::Unknown,
            }),
            8 => Category::ColorCorrection(match ColorCorrection::from_u8(parameter) {
                Some(v) => v,
                None => ColorCorrection::Unknown,
            }),
            9 => Category::Internal,
            10 => Category::Media(match Media::from_u8(parameter) {
                Some(v) => v,
                None => Media::Unknown,
            }),
            11 => Category::PTZ(match PTZ::from_u8(parameter) {
                Some(v) => v,
                None => PTZ::Unknown,
            }),
            12 => Category::Metadata(match Metadata::from_u8(parameter) {
                Some(v) => v,
                None => Metadata::Unknown,
            }),
            _ => Category::Unknown,
        }
    }

    pub fn get_category_id(&self) -> u8 {
        match self {
            Category::Lens(_) => 0,
            Category::Video(_) => 1,
            Category::Audio(_) => 2,
            Category::Output(_) => 3,
            Category::Display(_) => 4,
            Category::Tally(_) => 5,
            Category::Reference(_) => 6,
            Category::Configuration(_) => 7,
            Category::ColorCorrection(_) => 8,
            Category::Internal => 9,
            Category::Media(_) => 10,
            Category::PTZ(_) => 11,
            Category::Metadata(_) => 12,
            Category::Unknown => 255,
        }
    }

    pub fn get_parameter_id(&self) -> u8 {
        match self {
            Category::Lens(v) => v.to_u8().unwrap(),
            Category::Video(v) => v.to_u8().unwrap(),
            Category::Audio(v) => v.to_u8().unwrap(),
            Category::Output(v) => v.to_u8().unwrap(),
            Category::Display(v) => v.to_u8().unwrap(),
            Category::Tally(v) => v.to_u8().unwrap(),
            Category::Reference(v) => v.to_u8().unwrap(),
            Category::Configuration(v) => v.to_u8().unwrap(),
            Category::ColorCorrection(v) => v.to_u8().unwrap(),
            Category::Internal => 255,
            Category::Media(v) => v.to_u8().unwrap(),
            Category::PTZ(v) => v.to_u8().unwrap(),
            Category::Metadata(v) => v.to_u8().unwrap(),
            Category::Unknown => 255,
        }
    }
}

#[derive(Debug, PartialEq, Primitive)]
pub enum Lens {
    Focus = 0,
    InstantaneousAutofocus = 1,
    ApertureFStop = 2,
    ApertureNormalized = 3,
    ApertureOrdinal = 4,
    InstantaneousAutoAperture = 5,
    OpticalImageStabilization = 6,
    SetAbsoluteZoomMillimeter = 7,
    SetAbsoluteZoomNormalized = 8,
    SetAbsoluteZoomSpeed = 9,
    Unknown = 10,
}

#[derive(Debug, PartialEq, Primitive)]
pub enum Video {
    VideoMode = 0,
    GainISO = 1,
    ManualWhiteBalance = 2,
    SetAutoWhiteBalance = 3,
    RestoreAutoWhiteBalance = 4,
    ExposureNanoSeconds = 5,
    ExposureOrdinal = 6,
    DynamicRangeMode = 7,
    VideoSharpeningLevel = 8,
    RecordingFormat = 9,
    SetAutoExposureMode = 10,
    ShutterAngle = 11,
    ShutterSpeed = 12,
    GaindB = 13,
    GainISOValue = 14,
    DisplayLUT = 15,
    NDFilter = 16,
    Unknown = 17,
}

#[derive(Debug, PartialEq, Primitive)]
pub enum Audio {
    HeadphoneLevel = 0,
    HeadphoneProgramMix = 1,
    SpeakerLevel = 2,
    InputType = 3,
    InputLevels = 4,
    PhantomPower = 5,
    Unknown = 6,
}

#[derive(Debug, PartialEq, Primitive)]
pub enum Output {
    OverlayEnables = 0,
    FrameGuidesStyle = 1,
    FrameGuidesOpacity = 2,
    Overlays = 3,
    Unknown = 4,
}

#[derive(Debug, PartialEq, Primitive)]
pub enum Display {
    Brightness = 0,
    ExposureAndFocus = 1,
    ZebraLevel = 2,
    PeakingLevel = 3,
    ColorBarEnable = 4,
    FocusAssist = 5,
    ProgramReturnFeedEnable = 6,
    Unknown = 7,
}

#[derive(Debug, PartialEq, Primitive)]
pub enum Tally {
    TallyBrightness = 0,
    FrontTallyBrightness = 1,
    RearTallyBrightness = 2,
    Unknown = 8,
}

#[derive(Debug, PartialEq, Primitive)]
pub enum Reference {
    Source = 0,
    Offset = 1,
    Unknown = 2,
}

#[derive(Debug, PartialEq, Primitive)]
pub enum Configuration {
    RealTimeClock = 0,
    SystemLanguage = 1,
    Timezone = 2,
    Location = 3,
    Unknown = 4,
}

#[derive(Debug, PartialEq, Primitive)]
pub enum ColorCorrection {
    LiftAdjust = 0,
    GammaAdjust = 1,
    GainAdjust = 2,
    OffsetAdjust = 3,
    ContrastAdjust = 4,
    LumaMix = 5,
    ColorAdjust = 6,
    CorrectionResetDefault = 7,
    Unknown = 8,
}

#[derive(Debug, PartialEq, Primitive)]
pub enum Media {
    Codec = 0,
    TransportMode = 1,
    PlaybackControl = 2,
    StillCapture = 3,
    Unknown = 4,
}

#[derive(Debug, PartialEq, Primitive)]
pub enum PTZ {
    PanTiltVelocity = 0,
    MemoryPreset = 1,
    Unknown = 2,
}

#[derive(Debug, PartialEq, Primitive)]
pub enum Metadata {
    Reel = 0,
    SceneTags = 1,
    Scene = 2,
    Take = 3,
    GoodTake = 4,
    CameraID = 5,
    CameraOperator = 6,
    Director = 7,
    ProjectName = 8,
    LensType = 9,
    LensIris = 10,
    LensFocalLength = 11,
    LensDistance = 12,
    LensFilter = 13,
    SlateMode = 14,
    SlateTarget = 15,
    Unknown = 16,
}
