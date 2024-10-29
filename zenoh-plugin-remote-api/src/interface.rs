//
// Copyright (c) 2024 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//

use std::sync::Arc;

use base64::{prelude::BASE64_STANDARD, Engine};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ts_rs::TS;
use uuid::Uuid;
use zenoh::{
    key_expr::OwnedKeyExpr,
    qos::{CongestionControl, Priority, Reliability},
    query::{ConsolidationMode, Query, Reply, ReplyError},
    sample::{Sample, SampleKind},
};

// ██████  ███████ ███    ███  ██████  ████████ ███████      █████  ██████  ██     ███    ███ ███████ ███████ ███████  █████   ██████  ███████
// ██   ██ ██      ████  ████ ██    ██    ██    ██          ██   ██ ██   ██ ██     ████  ████ ██      ██      ██      ██   ██ ██       ██
// ██████  █████   ██ ████ ██ ██    ██    ██    █████       ███████ ██████  ██     ██ ████ ██ █████   ███████ ███████ ███████ ██   ███ █████
// ██   ██ ██      ██  ██  ██ ██    ██    ██    ██          ██   ██ ██      ██     ██  ██  ██ ██           ██      ██ ██   ██ ██    ██ ██
// ██   ██ ███████ ██      ██  ██████     ██    ███████     ██   ██ ██      ██     ██      ██ ███████ ███████ ███████ ██   ██  ██████  ███████

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct B64String(String);
impl From<String> for B64String {
    fn from(value: String) -> Self {
        B64String(value)
    }
}

impl B64String {
    pub fn b64_to_bytes(self) -> Result<Vec<u8>, base64::DecodeError> {
        BASE64_STANDARD.decode(self.0)
    }
}

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Serialize, Deserialize)]
pub enum RemoteAPIMsg {
    Data(DataMsg),
    Control(ControlMsg),
}

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Serialize, Deserialize)]
pub enum DataMsg {
    // Client -> SVR
    PublisherPut {
        id: Uuid,
        payload: B64String,
        attachment: Option<B64String>,
        encoding: Option<String>,
    },
    // SVR -> Client
    // Subscriber
    Sample(SampleWS, Uuid),
    // GetReply
    GetReply(ReplyWS),
    // Bidirectional
    Queryable(QueryableMsg),
}

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Serialize, Deserialize)]
pub enum QueryableMsg {
    // SVR -> Client
    // UUID of original queryable
    Query {
        queryable_uuid: Uuid,
        query: QueryWS,
    },
    // Client -> SVR
    Reply {
        reply: QueryReplyWS,
    },
}

//  ██████  ██████  ███    ██ ████████ ██████   ██████  ██          ███    ███ ███████ ███████ ███████  █████   ██████  ███████
// ██      ██    ██ ████   ██    ██    ██   ██ ██    ██ ██          ████  ████ ██      ██      ██      ██   ██ ██       ██
// ██      ██    ██ ██ ██  ██    ██    ██████  ██    ██ ██          ██ ████ ██ █████   ███████ ███████ ███████ ██   ███ █████
// ██      ██    ██ ██  ██ ██    ██    ██   ██ ██    ██ ██          ██  ██  ██ ██           ██      ██ ██   ██ ██    ██ ██
//  ██████  ██████  ██   ████    ██    ██   ██  ██████  ███████     ██      ██ ███████ ███████ ███████ ██   ██  ██████  ███████

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ControlMsg {
    // Session
    OpenSession,
    CloseSession,
    Session(Uuid),

    // Session Action Messages
    Get {
        #[ts(as = "OwnedKeyExprWrapper")]
        key_expr: OwnedKeyExpr,
        parameters: Option<String>,
        handler: HandlerChannel,
        id: Uuid,
        // Parameters
        #[serde(
            deserialize_with = "deserialize_consolidation_mode",
            serialize_with = "serialize_consolidation_mode",
            default
        )]
        #[ts(type = "number | undefined")]
        consolidation: Option<ConsolidationMode>,
        // timeout: Option<ConsolidationMode>,
        #[serde(
            deserialize_with = "deserialize_congestion_control",
            serialize_with = "serialize_congestion_control",
            default
        )]
        #[ts(type = "number | undefined")]
        congestion_control: Option<CongestionControl>,
        #[serde(
            deserialize_with = "deserialize_priority",
            serialize_with = "serialize_priority",
            default
        )]
        #[ts(type = "number | undefined")]
        priority: Option<Priority>,
        #[ts(type = "boolean | undefined")]
        express: Option<bool>,
        #[ts(type = "string | undefined")]
        encoding: Option<String>,
        #[ts(type = "string | undefined")]
        payload: Option<B64String>,
        #[ts(type = "string | undefined")]
        attachment: Option<B64String>,
    },
    GetFinished {
        id: Uuid,
    },
    Put {
        #[ts(as = "OwnedKeyExprWrapper")]
        key_expr: OwnedKeyExpr,
        payload: B64String,
        //
        #[ts(type = "string | undefined")]
        encoding: Option<String>,
        #[serde(
            deserialize_with = "deserialize_congestion_control",
            serialize_with = "serialize_congestion_control",
            default
        )]
        #[ts(type = "number | undefined")]
        congestion_control: Option<CongestionControl>,
        #[serde(
            deserialize_with = "deserialize_priority",
            serialize_with = "serialize_priority",
            default
        )]
        #[ts(type = "number | undefined")]
        priority: Option<Priority>,
        #[ts(type = "boolean | undefined")]
        express: Option<bool>,
        #[ts(type = "string | undefined")]
        attachment: Option<B64String>,
    },
    Delete {
        #[ts(as = "OwnedKeyExprWrapper")]
        key_expr: OwnedKeyExpr,
        //
        #[serde(
            deserialize_with = "deserialize_congestion_control",
            serialize_with = "serialize_congestion_control",
            default
        )]
        #[ts(type = "number | undefined")]
        congestion_control: Option<CongestionControl>,
        #[serde(
            deserialize_with = "deserialize_priority",
            serialize_with = "serialize_priority",
            default
        )]
        #[ts(type = "number | undefined")]
        priority: Option<Priority>,
        #[ts(type = "boolean | undefined")]
        express: Option<bool>,
        #[ts(type = "string | undefined")]
        attachment: Option<B64String>,
    },
    // Subscriber
    DeclareSubscriber {
        #[ts(as = "OwnedKeyExprWrapper")]
        key_expr: OwnedKeyExpr,
        handler: HandlerChannel,
        id: Uuid,
    },
    Subscriber(Uuid),
    UndeclareSubscriber(Uuid),

    // Publisher
    DeclarePublisher {
        #[ts(as = "OwnedKeyExprWrapper")]
        key_expr: OwnedKeyExpr,
        #[ts(type = "string | undefined")]
        encoding: Option<String>,
        #[serde(
            deserialize_with = "deserialize_congestion_control",
            serialize_with = "serialize_congestion_control",
            default
        )]
        #[ts(type = "number | undefined")]
        congestion_control: Option<CongestionControl>,
        #[serde(
            deserialize_with = "deserialize_priority",
            serialize_with = "serialize_priority",
            default
        )]
        #[ts(type = "number | undefined")]
        priority: Option<Priority>,
        #[serde(
            deserialize_with = "deserialize_reliability",
            serialize_with = "serialize_reliability",
            default
        )]
        #[ts(type = "number | undefined")]
        reliability: Option<Reliability>,
        #[ts(type = "boolean | undefined")]
        express: Option<bool>,
        id: Uuid,
    },
    UndeclarePublisher(Uuid),
    // Queryable
    DeclareQueryable {
        #[ts(as = "OwnedKeyExprWrapper")]
        key_expr: OwnedKeyExpr,
        id: Uuid,
        complete: bool,
    },
    UndeclareQueryable(Uuid),
}

fn deserialize_consolidation_mode<'de, D>(d: D) -> Result<Option<ConsolidationMode>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<u8>::deserialize(d) {
        Ok(Some(value)) => Ok(Some(match value {
            0u8 => ConsolidationMode::Auto,
            1u8 => ConsolidationMode::None,
            2u8 => ConsolidationMode::Monotonic,
            3u8 => ConsolidationMode::Latest,
            _ => {
                return Err(serde::de::Error::custom(format!(
                    "Value not valid for ConsolidationMode Enum {:?}",
                    value
                )))
            }
        })),
        Ok(None) => Ok(None),
        Err(err) => Err(serde::de::Error::custom(format!(
            "Value not valid for ConsolidationMode Enum {:?}",
            err
        ))),
    }
}

fn serialize_consolidation_mode<S>(
    consolidation_mode: &Option<ConsolidationMode>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match consolidation_mode {
        Some(c_mode) => s.serialize_u8(*c_mode as u8),
        None => s.serialize_none(),
    }
}

fn deserialize_congestion_control<'de, D>(d: D) -> Result<Option<CongestionControl>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<u8>::deserialize(d) {
        Ok(Some(value)) => Ok(Some(match value {
            0u8 => CongestionControl::Drop,
            1u8 => CongestionControl::Block,
            val => {
                return Err(serde::de::Error::custom(format!(
                    "Value not valid for CongestionControl Enum {:?}",
                    val
                )))
            }
        })),
        Ok(None) => Ok(None),
        val => Err(serde::de::Error::custom(format!(
            "Value not valid for CongestionControl Enum {:?}",
            val
        ))),
    }
}

fn serialize_congestion_control<S>(
    congestion_control: &Option<CongestionControl>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match congestion_control {
        Some(c_ctrl) => s.serialize_u8(*c_ctrl as u8),
        None => s.serialize_none(),
    }
}

fn deserialize_priority<'de, D>(d: D) -> Result<Option<Priority>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<u8>::deserialize(d) {
        Ok(Some(value)) => Ok(Some(match value {
            1u8 => Priority::RealTime,
            2u8 => Priority::InteractiveHigh,
            3u8 => Priority::InteractiveLow,
            4u8 => Priority::DataHigh,
            5u8 => Priority::Data,
            6u8 => Priority::DataLow,
            7u8 => Priority::Background,
            val => {
                return Err(serde::de::Error::custom(format!(
                    "Value not valid for Priority Enum {:?}",
                    val
                )))
            }
        })),
        Ok(None) => Ok(None),
        val => Err(serde::de::Error::custom(format!(
            "Value not valid for Priority Enum {:?}",
            val
        ))),
    }
}

fn serialize_priority<S>(priority: &Option<Priority>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match priority {
        Some(prio) => s.serialize_u8(*prio as u8),
        None => s.serialize_none(),
    }
}

fn deserialize_reliability<'de, D>(d: D) -> Result<Option<Reliability>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<u8>::deserialize(d) {
        Ok(Some(value)) => Ok(Some(match value {
            0u8 => Reliability::Reliable,
            1u8 => Reliability::BestEffort,
            val => {
                return Err(serde::de::Error::custom(format!(
                    "Value not valid for Reliability Enum {:?}",
                    val
                )))
            }
        })),
        Ok(None) => Ok(None),
        val => Err(serde::de::Error::custom(format!(
            "Value not valid for Reliability Enum {:?}",
            val
        ))),
    }
}

fn serialize_reliability<S>(reliability: &Option<Reliability>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match reliability {
        Some(prio) => s.serialize_u8(*prio as u8),
        None => s.serialize_none(),
    }
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub(crate) enum HandlerChannel {
    Fifo(usize),
    Ring(usize),
}

// ██     ██ ██████   █████  ██████  ██████  ███████ ██████  ███████
// ██     ██ ██   ██ ██   ██ ██   ██ ██   ██ ██      ██   ██ ██
// ██  █  ██ ██████  ███████ ██████  ██████  █████   ██████  ███████
// ██ ███ ██ ██   ██ ██   ██ ██      ██      ██      ██   ██      ██
//  ███ ███  ██   ██ ██   ██ ██      ██      ███████ ██   ██ ███████

// Wrapper to get OwnerKeyExpr to play with TS
#[allow(dead_code)] // To allow OwnedKeyExpr to be converted to String
#[derive(Debug, Deserialize, TS)]
#[serde(from = "String")]
pub struct OwnedKeyExprWrapper(Arc<str>);

impl From<String> for OwnedKeyExprWrapper {
    fn from(s: String) -> Self {
        OwnedKeyExprWrapper(s.into())
    }
}

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryWS {
    query_uuid: Uuid,
    #[ts(as = "OwnedKeyExprWrapper")]
    key_expr: OwnedKeyExpr,
    parameters: String,
    encoding: Option<String>,
    #[ts(type = "string | undefined")]
    attachment: Option<B64String>,
    #[ts(type = "string | undefined")]
    payload: Option<B64String>,
}

impl From<(&Query, Uuid)> for QueryWS {
    fn from((q, uuid): (&Query, Uuid)) -> Self {
        let payload = q
            .payload()
            .map(|x| x.to_bytes().to_vec())
            .map(|vec_bytes| BASE64_STANDARD.encode(vec_bytes).into());
        let attachment: Option<B64String> = q
            .attachment()
            .map(|x| x.to_bytes().to_vec())
            .map(|vec_bytes| BASE64_STANDARD.encode(vec_bytes).into());

        QueryWS {
            query_uuid: uuid,
            key_expr: q.key_expr().to_owned().into(),
            parameters: q.parameters().to_string(),
            encoding: q.encoding().map(|x| x.to_string()),
            attachment,
            payload,
        }
    }
}

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ReplyWS {
    pub query_uuid: Uuid,
    pub result: Result<SampleWS, ReplyErrorWS>,
}

impl From<(Reply, Uuid)> for ReplyWS {
    fn from((reply, uuid): (Reply, Uuid)) -> Self {
        match reply.result() {
            Ok(sample) => {
                let sample_ws = SampleWS::from(sample);
                ReplyWS {
                    query_uuid: uuid,
                    result: Ok(sample_ws),
                }
            }
            Err(err) => {
                let error_ws = ReplyErrorWS::from(err);

                ReplyWS {
                    query_uuid: uuid,
                    result: Err(error_ws),
                }
            }
        }
    }
}

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryReplyWS {
    pub query_uuid: Uuid,
    pub result: QueryReplyVariant,
}

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Serialize, Deserialize)]
pub enum QueryReplyVariant {
    Reply {
        #[ts(as = "OwnedKeyExprWrapper")]
        key_expr: OwnedKeyExpr,
        payload: B64String,
    },
    ReplyErr {
        payload: B64String,
    },
    ReplyDelete {
        #[ts(as = "OwnedKeyExprWrapper")]
        key_expr: OwnedKeyExpr,
    },
}

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ReplyErrorWS {
    pub(crate) payload: B64String,
    pub(crate) encoding: String,
}

impl From<ReplyError> for ReplyErrorWS {
    fn from(r_e: ReplyError) -> Self {
        let z_bytes: Vec<u8> = r_e.payload().to_bytes().to_vec();

        ReplyErrorWS {
            payload: BASE64_STANDARD.encode(z_bytes).into(),
            encoding: r_e.encoding().to_string(),
        }
    }
}

impl From<&ReplyError> for ReplyErrorWS {
    fn from(r_e: &ReplyError) -> Self {
        let z_bytes: Vec<u8> = r_e.payload().to_bytes().to_vec();

        ReplyErrorWS {
            payload: base64::prelude::BASE64_STANDARD.encode(z_bytes).into(),
            encoding: r_e.encoding().to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SampleWS {
    #[ts(as = "OwnedKeyExprWrapper")]
    pub(crate) key_expr: OwnedKeyExpr,
    pub(crate) value: B64String,
    pub(crate) kind: SampleKindWS,
    pub(crate) encoding: String,
    pub(crate) timestamp: Option<String>,
    pub(crate) congestion_control: u8,
    pub(crate) priority: u8,
    pub(crate) express: bool,
    pub(crate) attachement: Option<B64String>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum SampleKindWS {
    Put = 0,
    Delete = 1,
}

impl From<SampleKind> for SampleKindWS {
    fn from(sk: SampleKind) -> Self {
        match sk {
            SampleKind::Put => SampleKindWS::Put,
            SampleKind::Delete => SampleKindWS::Delete,
        }
    }
}

impl From<&Sample> for SampleWS {
    fn from(s: &Sample) -> Self {
        let z_bytes: Vec<u8> = s.payload().to_bytes().to_vec();

        SampleWS {
            key_expr: s.key_expr().to_owned().into(),
            value: BASE64_STANDARD.encode(z_bytes).into(),
            kind: s.kind().into(),
            timestamp: s.timestamp().map(|x| x.to_string()),
            priority: s.priority() as u8,
            congestion_control: s.congestion_control() as u8,
            encoding: s.encoding().to_string(),
            express: s.express(),
            attachement: s
                .attachment()
                .map(|x| x.to_bytes().to_vec())
                .map(|z_bytes| BASE64_STANDARD.encode(z_bytes).into()),
        }
    }
}

impl From<Sample> for SampleWS {
    fn from(s: Sample) -> Self {
        SampleWS::from(&s)
    }
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use uuid::Uuid;
    use zenoh::key_expr::KeyExpr;

    use super::*;

    #[test]
    fn test_b64_serializing() {
        let bytes: Vec<u8> = std::iter::repeat(245).take(100).collect();

        let b64_string = BASE64_STANDARD.encode(bytes.clone());

        #[derive(Debug, Serialize, Deserialize)]
        struct RawBytes {
            bytes: Vec<u8>,
        }
        #[derive(Debug, Serialize, Deserialize)]
        struct B64Encoded {
            b64_string: String,
        }

        let json_bytes = serde_json::to_string(&RawBytes { bytes }).unwrap();
        let json_b64 = serde_json::to_string(&B64Encoded { b64_string }).unwrap();
        assert!(json_b64.len() < json_bytes.len())
    }

    #[test]
    fn serialize_messages() {
        let json: String =
            serde_json::to_string(&RemoteAPIMsg::Control(ControlMsg::OpenSession)).unwrap();
        assert_eq!(json, r#"{"Control":"OpenSession"}"#);

        let uuid = Uuid::from_str("a2663bb1-128c-4dd3-a42b-d1d3337e2e51").unwrap();

        let json: String =
            serde_json::to_string(&RemoteAPIMsg::Control(ControlMsg::Session(uuid))).unwrap();
        assert_eq!(
            json,
            r#"{"Control":{"Session":"a2663bb1-128c-4dd3-a42b-d1d3337e2e51"}}"#
        );

        let json: String =
            serde_json::to_string(&RemoteAPIMsg::Control(ControlMsg::CloseSession)).unwrap();
        assert_eq!(json, r#"{"Control":"CloseSession"}"#);

        let key_expr: OwnedKeyExpr = KeyExpr::new("demo/test").unwrap().to_owned().into();

        let _sample_ws = SampleWS {
            key_expr: key_expr.clone(),
            value: BASE64_STANDARD.encode(vec![1, 2, 3]).into(),
            kind: SampleKindWS::Put,
            encoding: "zenoh/bytes".into(),
            timestamp: None,
            priority: 1,
            congestion_control: 1,
            express: false,
            attachement: None,
        };

        let sample_ws = SampleWS {
            key_expr,
            value: BASE64_STANDARD.encode(vec![1, 2, 3]).into(),
            kind: SampleKindWS::Put,
            encoding: "zenoh/bytes".into(),
            timestamp: None,
            priority: 1,
            congestion_control: 1,
            express: false,
            attachement: None,
        };
        let _json: String = serde_json::to_string(&DataMsg::Sample(sample_ws, uuid)).unwrap();
    }
}