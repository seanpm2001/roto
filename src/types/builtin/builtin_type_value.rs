//------------ BuiltinTypeValue -------------------------------------------

// The built-in types

use std::fmt::Display;
use std::sync::Arc;

use serde::Serialize;

use crate::compile::CompileError;
use crate::traits::RotoType;
use crate::types::collections::BytesRecord;
use crate::types::constant_enum::EnumVariant;
use crate::types::lazytypedef::{PeerDownNotification, PeerUpNotification, RouteMonitoring};

use super::super::collections::List;
use super::super::typedef::TypeDef;
use super::super::typevalue::TypeValue;

use super::{
    AsPath, Asn, AtomicAggregator, Boolean, Community, HexLiteral, Hop,
    IntegerLiteral, IpAddress, LocalPref, MultiExitDisc, NextHop, OriginType,
    Prefix, PrefixLength, BgpUpdateMessage, RawRouteWithDeltas, RouteStatus,
    StringLiteral, U32, U8, U16,
};

#[derive(Debug, Eq, Clone, Hash, PartialEq, Serialize)]
#[serde(untagged)]
pub enum BuiltinTypeValue {
    U32(U32),                       // scalar
    U16(U16),                       // scalar
    U8(U8),                         // scalar
    IntegerLiteral(IntegerLiteral), // scalar
    StringLiteral(StringLiteral),   // scalar
    Boolean(Boolean),               // scalar
    HexLiteral(HexLiteral),         // scalar
    IpAddress(IpAddress),           // scalar
    Prefix(Prefix),                 // scalar
    PrefixLength(PrefixLength),     // scalar
    LocalPref(LocalPref),               // scalar
    AtomicAggregator(AtomicAggregator), // scalar
    NextHop(NextHop),                   // scalar
    MultiExitDisc(MultiExitDisc),       // scalar
    RouteStatus(RouteStatus),           // scalar
    Community(Community),           // scalar
    Communities(List),              // vector
    Asn(Asn),                       // scalar
    AsPath(AsPath),                 // vector
    Hop(Hop),                       // read-only scalar
    OriginType(OriginType),         // scalar
    Route(RawRouteWithDeltas),      // vector
    // A read-only enum variant for capturing constants
    ConstU8EnumVariant(EnumVariant<u8>),
    ConstU16EnumVariant(EnumVariant<u16>),
    ConstU32EnumVariant(EnumVariant<u32>),
    // Used for filtering on the properties of the whole message,
    // not taking into account any individual prefixes.
    BgpUpdateMessage(Arc<BgpUpdateMessage>),  // scalar
    BmpRouteMonitoringMessage(Arc<BytesRecord<RouteMonitoring>>),
    BmpPeerUpNotification(Arc<BytesRecord<PeerUpNotification>>),
    BmpPeerDownNotification(Arc<BytesRecord<PeerDownNotification>>)
}

impl BuiltinTypeValue {
    pub fn create_instance(
        ty: TypeDef,
        value: impl Into<BuiltinTypeValue>,
    ) -> Result<TypeValue, CompileError> {
        let var = match ty {
            TypeDef::U32 => {
                if let BuiltinTypeValue::U32(v) = value.into() {
                    BuiltinTypeValue::U32(v)
                } else {
                    return Err("Not a U32".into());
                }
            }
            TypeDef::U8 => {
                if let BuiltinTypeValue::U8(v) = value.into() {
                    BuiltinTypeValue::U8(v)
                } else {
                    return Err("Not a U8".into());
                }
            }
            TypeDef::IntegerLiteral => {
                if let BuiltinTypeValue::IntegerLiteral(v) = value.into() {
                    BuiltinTypeValue::IntegerLiteral(v)
                } else {
                    return Err("Not an IntegerLiteral".into());
                }
            }
            TypeDef::StringLiteral => {
                if let BuiltinTypeValue::StringLiteral(v) = value.into() {
                    BuiltinTypeValue::StringLiteral(v)
                } else {
                    return Err("Not a StringLiteral".into());
                }
            }
            TypeDef::PrefixLength => {
                if let BuiltinTypeValue::PrefixLength(v) = value.into() {
                    BuiltinTypeValue::PrefixLength(v)
                } else {
                    return Err("Not a PrefixLength".into());
                }
            }
            TypeDef::HexLiteral => {
                if let BuiltinTypeValue::HexLiteral(v) = value.into() {
                    BuiltinTypeValue::HexLiteral(v)
                } else {
                    return Err("Not a HexLiteral".into());
                }
            }
            TypeDef::Prefix => {
                if let BuiltinTypeValue::Prefix(v) = value.into() {
                    BuiltinTypeValue::Prefix(v)
                } else {
                    return Err("Not a Prefix".into());
                }
            }
            TypeDef::IpAddress => {
                if let BuiltinTypeValue::IpAddress(v) = value.into() {
                    BuiltinTypeValue::IpAddress(v)
                } else {
                    return Err("Not an IP address".into());
                }
            }
            TypeDef::Asn => {
                if let BuiltinTypeValue::Asn(v) = value.into() {
                    BuiltinTypeValue::Asn(v)
                } else {
                    return Err("Not an ASN".into());
                }
            }
            TypeDef::AsPath => {
                if let BuiltinTypeValue::AsPath(v) = value.into() {
                    BuiltinTypeValue::AsPath(v)
                } else {
                    return Err("Not an AS Path".into());
                }
            }
            TypeDef::Community => {
                if let BuiltinTypeValue::Community(v) = value.into() {
                    BuiltinTypeValue::Community(v)
                } else {
                    return Err("Not a community".into());
                }
            }
            TypeDef::Route => {
                if let BuiltinTypeValue::Route(v) = value.into() {
                    BuiltinTypeValue::Route(v)
                } else {
                    return Err("Not a route".into());
                }
            }
            TypeDef::RouteStatus => {
                if let BuiltinTypeValue::RouteStatus(v) = value.into() {
                    BuiltinTypeValue::RouteStatus(v)
                } else {
                    return Err("Not a route status".into());
                }
            }
            _ => return Err("Not a primitive type".into()),
        };
        Ok(var.into())
    }

    pub(crate) fn into_type(
        self,
        ty: &TypeDef,
    ) -> Result<TypeValue, CompileError> {
        match self {
            BuiltinTypeValue::U32(v) => v.into_type(ty),
            BuiltinTypeValue::U16(v) => v.into_type(ty),
            BuiltinTypeValue::U8(v) => v.into_type(ty),
            BuiltinTypeValue::ConstU8EnumVariant(v) => v.into_type(ty),
            BuiltinTypeValue::ConstU16EnumVariant(v) => v.into_type(ty),
            BuiltinTypeValue::ConstU32EnumVariant(v) => v.into_type(ty),
            BuiltinTypeValue::IntegerLiteral(v) => v.into_type(ty),
            BuiltinTypeValue::StringLiteral(v) => v.into_type(ty),
            BuiltinTypeValue::Prefix(v) => v.into_type(ty),
            BuiltinTypeValue::PrefixLength(v) => v.into_type(ty),
            BuiltinTypeValue::Community(v) => v.into_type(ty),
            BuiltinTypeValue::Communities(_v) => {
                Err("Can't convert from communities list".into())
            }
            BuiltinTypeValue::IpAddress(v) => v.into_type(ty),
            BuiltinTypeValue::AsPath(v) => v.into_type(ty),
            BuiltinTypeValue::Hop(h) => h.into_type(ty),
            BuiltinTypeValue::OriginType(v) => v.into_type(ty),
            BuiltinTypeValue::Route(r) => r.into_type(ty),
            BuiltinTypeValue::BgpUpdateMessage(_raw) => Err(CompileError::from(
                "Cannot convert raw BGP message into any other type.",
            )),
            BuiltinTypeValue::BmpRouteMonitoringMessage(_raw) => Err(CompileError::from(
                "Cannot convert raw BMP Route Monitoring message into any other type.",
            )),
            BuiltinTypeValue::BmpPeerUpNotification(_raw) => Err(CompileError::from(
                "Cannot convert raw BMP Peer Up Notification into any other type.",
            )),
            BuiltinTypeValue::BmpPeerDownNotification(_raw) => Err(CompileError::from(
                "Cannot convert raw BMP Peer Down Notification into any other type.",
            )),
            BuiltinTypeValue::RouteStatus(v) => v.into_type(ty),
            BuiltinTypeValue::Boolean(v) => v.into_type(ty),
            BuiltinTypeValue::HexLiteral(v) => v.into_type(ty),
            BuiltinTypeValue::Asn(v) => v.into_type(ty),
            BuiltinTypeValue::LocalPref(v) => v.into_type(ty),
            BuiltinTypeValue::AtomicAggregator(v) => v.into_type(ty),
            BuiltinTypeValue::NextHop(v) => v.into_type(ty),
            BuiltinTypeValue::MultiExitDisc(v) => v.into_type(ty),
        }
    }
}

// These From impls allow the user to use the create_instance function with
// simple types like u32, u8, etc. (without the nested variants).

impl From<Asn> for BuiltinTypeValue {
    fn from(val: Asn) -> Self {
        BuiltinTypeValue::Asn(val)
    }
}

impl From<u32> for BuiltinTypeValue {
    fn from(val: u32) -> Self {
        BuiltinTypeValue::U32(U32(val))
    }
}

impl From<U16> for BuiltinTypeValue {
    fn from(val: U16) -> Self {
        BuiltinTypeValue::U16(val)
    }
}

impl From<HexLiteral> for BuiltinTypeValue {
    fn from(value: HexLiteral) -> Self {
        BuiltinTypeValue::HexLiteral(value)
    }
}

impl From<i64> for BuiltinTypeValue {
    fn from(val: i64) -> Self {
        BuiltinTypeValue::IntegerLiteral(IntegerLiteral(val))
    }
}

impl From<std::net::IpAddr> for BuiltinTypeValue {
    fn from(val: std::net::IpAddr) -> Self {
        BuiltinTypeValue::IpAddress(IpAddress(val))
    }
}

impl From<routecore::addr::Prefix> for BuiltinTypeValue {
    fn from(val: routecore::addr::Prefix) -> Self {
        BuiltinTypeValue::Prefix(Prefix(val))
    }
}

impl From<crate::types::builtin::primitives::IpAddress> for BuiltinTypeValue {
    fn from(value: crate::types::builtin::primitives::IpAddress) -> Self {
        BuiltinTypeValue::IpAddress(value)
    }
}

// impl TryFrom<&'_ str> for BuiltinTypeValue {
//     type Error = CompileError;

//     fn try_from(val: &'_ str) -> Result<Self, Self::Error> {
//         match val {
//             "U32" => Ok(BuiltinTypeValue::U32(U32(None))),
//             "U8" => Ok(BuiltinTypeValue::U8(U8(None))),
//             "IntegerLiteral" => {
//                 Ok(BuiltinTypeValue::IntegerLiteral(IntegerLiteral(None)))
//             }
//             "PrefixLengthLiteral" => {
//                 Ok(BuiltinTypeValue::PrefixLength(PrefixLength(None)))
//             }
//             "Boolean" => Ok(BuiltinTypeValue::Boolean(Boolean(None))),
//             "Prefix" => Ok(BuiltinTypeValue::Prefix(Prefix(None))),
//             "PrefixLength" => {
//                 Ok(BuiltinTypeValue::PrefixLength(PrefixLength(None)))
//             }
//             "Community" => Ok(BuiltinTypeValue::Community(Community(None))),
//             "IpAddress" => Ok(BuiltinTypeValue::IpAddress(IpAddress(None))),
//             "Asn" => Ok(BuiltinTypeValue::Asn(Asn(None))),
//             "AsPath" => Ok(BuiltinTypeValue::AsPath(AsPath(None))),
//             "Route" => Ok(BuiltinTypeValue::Route(None)),
//             "RouteStatus" => {
//                 Ok(BuiltinTypeValue::RouteStatus(RouteStatus::Empty))
//             }
//             _ => Err(format!("Unknown type: {}", val).into()),
//         }
//     }
// }

// impl TryFrom<&TypeDef> for BuiltinTypeValue {
//     type Error = CompileError;

//     fn try_from(ty: &TypeDef) -> Result<Self, Self::Error> {
//         match ty {
//             TypeDef::U32 => Ok(BuiltinTypeValue::U32(U32(None))),
//             TypeDef::U8 => Ok(BuiltinTypeValue::U8(U8(None))),
//             TypeDef::IntegerLiteral => {
//                 Ok(BuiltinTypeValue::IntegerLiteral(IntegerLiteral(None)))
//             }
//             TypeDef::Boolean => Ok(BuiltinTypeValue::Boolean(Boolean(None))),
//             TypeDef::Prefix => Ok(BuiltinTypeValue::Prefix(Prefix(None))),
//             TypeDef::PrefixLength => {
//                 Ok(BuiltinTypeValue::PrefixLength(PrefixLength(None)))
//             }
//             TypeDef::Community => {
//                 Ok(BuiltinTypeValue::Community(Community(None)))
//             }
//             TypeDef::IpAddress => {
//                 Ok(BuiltinTypeValue::IpAddress(IpAddress(None)))
//             }
//             TypeDef::Asn => Ok(BuiltinTypeValue::Asn(Asn(None))),
//             TypeDef::AsPath => Ok(BuiltinTypeValue::AsPath(AsPath(None))),
//             TypeDef::Route => Ok(BuiltinTypeValue::Route(None)),
//             TypeDef::RouteStatus => {
//                 Ok(BuiltinTypeValue::RouteStatus(RouteStatus::Empty))
//             }
//             _ => Err(format!("Unknown type: {:?}", ty).into()),
//         }
//     }
// }

impl Display for BuiltinTypeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BuiltinTypeValue::U32(v) => write!(f, "{} (U32)", v),
            BuiltinTypeValue::U16(v) => write!(f, "{} (U16)", v),
            BuiltinTypeValue::U8(v) => write!(f, "{} (U8)", v),
            BuiltinTypeValue::IntegerLiteral(v) => {
                write!(f, "{} (Integer)", v)
            }
            BuiltinTypeValue::StringLiteral(v) => {
                write!(f, "{} (String)", v)
            }
            BuiltinTypeValue::ConstU8EnumVariant(v) => {
                write!(f, "{} (Const U8 Enum Variant)", v)
            }
            BuiltinTypeValue::ConstU16EnumVariant(v) => {
                write!(f, "{} (Const U16 Enum Variant)", v)
            }
            BuiltinTypeValue::ConstU32EnumVariant(v) => {
                write!(f, "{} (Const U32 Enum Variant)", v)
            }
            BuiltinTypeValue::Prefix(v) => write!(f, "{} (Prefix)", v),
            BuiltinTypeValue::PrefixLength(v) => {
                write!(f, "{} (Prefix Length)", v)
            }
            BuiltinTypeValue::Community(v) => write!(f, "{} (Community)", v),
            BuiltinTypeValue::Communities(v) => {
                write!(f, "{:?} (Communities)", v)
            }
            BuiltinTypeValue::IpAddress(v) => write!(f, "{} (IP Address)", v),
            BuiltinTypeValue::Asn(v) => write!(f, "{} (ASN)", v),
            BuiltinTypeValue::AsPath(v) => {
                write!(f, "{} (AS Path)", v)
            }
            BuiltinTypeValue::Hop(h) => {
                write!(f, "{} (Hop)", h)
            }
            BuiltinTypeValue::OriginType(v) => {
                write!(f, "{} (Origin Type)", v)
            }
            BuiltinTypeValue::Route(r) => write!(f, "{} (Route)", r),
            BuiltinTypeValue::BgpUpdateMessage(raw) => {
                write!(f, "{:X?} (RawBgpMessage)", **raw)
            }
            BuiltinTypeValue::BmpRouteMonitoringMessage(raw) => {
                write!(f, "{:X?} (BmpRouteMonitoringMessage)", **raw)
            }
            BuiltinTypeValue::BmpPeerUpNotification(raw) => {
                write!(f, "{:X?} (BmpPeerUpNotification)", **raw)
            }
            BuiltinTypeValue::BmpPeerDownNotification(raw) => {
                write!(f, "{:X?} (BmpPeerDownNotification)", **raw)
            }
            BuiltinTypeValue::RouteStatus(v) => {
                write!(f, "{} (Route Status)", v)
            }
            BuiltinTypeValue::Boolean(v) => write!(f, "{} (Boolean)", v),
            BuiltinTypeValue::HexLiteral(v) => {
                write!(f, "{} (Hex)", v)
            }
            BuiltinTypeValue::LocalPref(v) => {
                write!(f, "{} (Local Preference)", v)
            }
            BuiltinTypeValue::AtomicAggregator(v) => {
                write!(f, "{} (Atomic Aggregator)", v)
            }
            BuiltinTypeValue::NextHop(v) => write!(f, "{} (Next Hop)", v),
            BuiltinTypeValue::MultiExitDisc(v) => {
                write!(f, "{} (Multi Exit Discriminator)", v)
            }
        }
    }
}
