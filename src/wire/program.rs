//! Types used for the program/ endpoint

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use validator::Validate;

use crate::wire::event::EventPayloadDescriptor;
use crate::wire::interval::IntervalPeriod;
use crate::wire::report::ReportPayloadDescriptor;
use crate::wire::target::{TargetLabel, TargetMap};
use crate::wire::{DateTime, Duration};

pub type Programs = Vec<Program>;

// TODO: This should actually be split into two flattened structs... one for get and one for put/post
/// Provides program specific metadata from VTN to VEN.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Program {
    /// VTN provisioned on object creation.
    ///
    /// URL safe VTN assigned object ID.
    pub id: ProgramId,
    /// VTN provisioned on object creation.
    ///
    /// datetime in ISO 8601 format
    pub created_date_time: DateTime,
    /// VTN provisioned on object modification.
    ///
    /// datetime in ISO 8601 format
    pub modification_date_time: DateTime,
    #[serde(flatten)]
    pub content: NewProgram,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[skip_serializing_none]
#[serde(rename_all = "camelCase")]
pub struct NewProgram {
    /// Used as discriminator, e.g. notification.object
    ///
    /// VTN provisioned on object creation.
    // TODO: Maybe remove this? It is more part of the enum containing this
    pub object_type: Option<ProgramObjectType>,
    /// Short name to uniquely identify program.
    pub program_name: ProgramName,
    /// Long name of program for human readability.
    pub program_long_name: Option<String>,
    /// Short name of energy retailer providing the program.
    pub retailer_name: Option<String>,
    /// Long name of energy retailer for human readability.
    pub retailer_long_name: Option<String>,
    /// A program defined categorization.
    pub program_type: Option<String>,
    /// Alpha-2 code per ISO 3166-1.
    pub country: Option<String>,
    /// Coding per ISO 3166-2. E.g. state in US.
    pub principal_subdivision: Option<String>,
    /// duration in ISO 8601 format
    ///
    /// Number of hours different from UTC for the standard time applicable to the program.
    // TODO: aaaaaah why???
    pub time_zone_offset: Option<Duration>,
    pub interval_period: Option<IntervalPeriod>,
    /// A list of programDescriptions
    pub program_descriptions: Option<Vec<ProgramDescription>>,
    /// True if events are fixed once transmitted.
    pub binding_events: Option<bool>,
    /// True if events have been adapted from a grid event.
    pub local_price: Option<bool>,
    /// A list of payloadDescriptors.
    pub payload_descriptors: Option<Vec<PayloadDescriptor>>,
    /// A list of valuesMap objects.
    pub targets: Option<TargetMap>,
}

// TODO enforce constraints:
//     objectID:
//         type: string
//         pattern: /^[a-zA-Z0-9_-]*$/
//         minLength: 1
//         maxLength: 128
//         description: URL safe VTN assigned object ID.
//         example: object-999
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Eq)]
pub struct ProgramId(pub String);

// TODO: enforce length requirement
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProgramName(String);

impl ProgramName {
    pub fn new(name: String) -> Self {
        Self(name)
    }
}

/// Used as discriminator, e.g. notification.object
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProgramObjectType {
    #[default]
    Program,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ProgramDescription {
    /// A human or machine readable program description
    #[serde(rename = "URL")]
    pub url: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "objectType", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PayloadDescriptor {
    EventPayloadDescriptor(EventPayloadDescriptor),
    ReportPayloadDescriptor(ReportPayloadDescriptor),
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    target_type: Option<TargetLabel>,
    target_values: Option<Vec<String>>,
    #[serde(default)]
    skip: u32,
    #[validate(range(max = 50))]
    limit: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_parses() {
        let example = r#"[
                  {
                    "id": "object-999",
                    "createdDateTime": "2023-06-15T09:30:00Z",
                    "modificationDateTime": "2023-06-15T09:30:00Z",
                    "objectType": "PROGRAM",
                    "programName": "ResTOU",
                    "programLongName": "Residential Time of Use-A",
                    "retailerName": "ACME",
                    "retailerLongName": "ACME Electric Inc.",
                    "programType": "PRICING_TARIFF",
                    "country": "US",
                    "principalSubdivision": "CO",
                    "timeZoneOffset": "PT1H",
                    "intervalPeriod": {
                      "start": "2023-06-15T09:30:00Z",
                      "duration": "PT1H",
                      "randomizeStart": "PT1H"
                    },
                    "programDescriptions": null,
                    "bindingEvents": false,
                    "localPrice": false,
                    "payloadDescriptors": null,
                    "targets": null
                  }
                ]"#;

        let parsed = serde_json::from_str::<Programs>(example).unwrap();

        let expected = vec![Program {
            id: ProgramId("object-999".into()),
            created_date_time: DateTime("2023-06-15T09:30:00Z".into()),
            modification_date_time: DateTime("2023-06-15T09:30:00Z".into()),
            content: NewProgram {
                object_type: Some(ProgramObjectType::Program),
                program_name: ProgramName("ResTOU".into()),
                program_long_name: Some("Residential Time of Use-A".into()),
                retailer_name: Some("ACME".into()),
                retailer_long_name: Some("ACME Electric Inc.".into()),
                program_type: Some("PRICING_TARIFF".into()),
                country: Some("US".into()),
                principal_subdivision: Some("CO".into()),
                time_zone_offset: Some(Duration("PT1H".into())),
                interval_period: Some(IntervalPeriod {
                    start: DateTime("2023-06-15T09:30:00Z".into()),
                    duration: Some(Duration("PT1H".into())),
                    randomize_start: Some(Duration("PT1H".into())),
                }),
                program_descriptions: None,
                binding_events: Some(false),
                local_price: Some(false),
                payload_descriptors: None,
                targets: None,
            },
        }];

        assert_eq!(expected, parsed);
    }

    #[test]
    fn parses_minimal() {
        let example = r#"{"programName":"test"}"#;

        assert_eq!(
            serde_json::from_str::<NewProgram>(example).unwrap(),
            NewProgram {
                object_type: None,
                program_name: ProgramName("test".to_string()),
                program_long_name: None,
                retailer_name: None,
                retailer_long_name: None,
                program_type: None,
                country: None,
                principal_subdivision: None,
                time_zone_offset: None,
                interval_period: None,
                program_descriptions: None,
                binding_events: None,
                local_price: None,
                payload_descriptors: None,
                targets: None,
            }
        );
    }
}
