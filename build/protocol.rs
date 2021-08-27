use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlackmagicCameraProtocol {
    pub information: Information,
    pub groups: Vec<Group>,
    #[serde(rename = "bluetooth_services")]
    pub bluetooth_services: Vec<BluetoothService>,
}

impl BlackmagicCameraProtocol {
    pub fn new() -> Result<BlackmagicCameraProtocol, std::io::Error> {
        let data = include_str!("../PROTOCOL.json");
        let cfg: BlackmagicCameraProtocol = serde_json::from_str(&data)?;
        Ok(cfg)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Information {
    pub readme: String,
    pub source: String,
    pub git: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub name: String,
    #[serde(rename = "normalized_name")]
    pub normalized_name: String,
    pub id: i64,
    pub parameters: Vec<Parameter>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub id: i64,
    pub group: String,
    #[serde(rename = "group_id")]
    pub group_id: i64,
    pub parameter: String,
    #[serde(rename = "normalized_parameter")]
    pub normalized_parameter: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub index: Vec<String>,
    pub interpretation: Option<String>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BluetoothService {
    pub name: String,
    #[serde(rename = "normalized_name")]
    pub normalized_name: String,
    pub uuid: uuid::Uuid,
    pub characteristics: Vec<Characteristic>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Characteristic {
    pub name: String,
    #[serde(rename = "normalized_name")]
    pub normalized_name: String,
    pub uuid: uuid::Uuid,
    pub description: Option<String>,
    pub decription: Option<String>,
}
