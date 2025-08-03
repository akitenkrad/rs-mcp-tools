pub mod io_client;
pub mod sse_client;

use openai_tools::common::{
    parameters::{ParameterProperty as OpenAIParameterProperty, Parameters as OpenAIParameters},
    tool::Tool as OpenAITool,
};
use serde_json::{Map, Value};
use std::path::PathBuf;
use url::Url;

pub trait IntoParameters {
    fn into_parameters(&self) -> Vec<ParameterSetting>;
}

#[derive(Debug, Clone, Default)]
pub struct ParameterSetting {
    pub name: String,
    pub type_name: String,
    pub description: String,
    pub enum_values: Option<Vec<String>>,
}

impl ParameterSetting {
    pub fn new<T: AsRef<str>>(
        name: T,
        type_name: T,
        description: T,
        enum_values: Option<Vec<T>>,
    ) -> Self {
        Self {
            name: name.as_ref().to_string(),
            type_name: type_name.as_ref().to_string(),
            description: description.as_ref().to_string(),
            enum_values: enum_values
                .map(|v| v.into_iter().map(|s| s.as_ref().to_string()).collect()),
        }
    }
}

impl From<ParameterSetting> for OpenAIParameterProperty {
    fn from(param: ParameterSetting) -> Self {
        OpenAIParameterProperty {
            type_name: param.type_name,
            description: Some(param.description),
            enum_values: param.enum_values,
        }
    }
}

impl IntoParameters for OpenAIParameters {
    fn into_parameters(&self) -> Vec<ParameterSetting> {
        self.properties
            .clone()
            .into_iter()
            .map(|(name, prop)| ParameterSetting {
                name,
                type_name: prop.type_name,
                description: prop.description.unwrap_or_default(),
                enum_values: prop
                    .enum_values
                    .map(|v| v.into_iter().map(|s| s.to_string()).collect()),
            })
            .collect()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParameterSetting>,
    pub sse_url: Option<Url>,
    pub io_path: Option<PathBuf>,
    pub arguments: Option<Map<String, Value>>,
}

impl Tool {
    pub fn with_io_transport(
        name: String,
        description: String,
        parameters: Vec<ParameterSetting>,
        io_path: PathBuf,
    ) -> Self {
        // Ensure the io_path is valid
        assert!(
            io_path.exists(),
            "No such file or directory: {}",
            io_path.display()
        );
        Self {
            name,
            description,
            parameters,
            io_path: Some(io_path),
            ..Default::default()
        }
    }
    pub fn with_sse_transport(
        name: String,
        description: String,
        parameters: Vec<ParameterSetting>,
        sse_url: Url,
    ) -> Self {
        Self {
            name,
            description,
            parameters,
            sse_url: Some(sse_url),
            ..Default::default()
        }
    }
}

impl From<Tool> for OpenAITool {
    fn from(tool: Tool) -> Self {
        let params = tool
            .parameters
            .into_iter()
            .map(|param| (param.name.clone(), OpenAIParameterProperty::from(param)))
            .collect::<Vec<(String, OpenAIParameterProperty)>>();
        OpenAITool::function(tool.name, tool.description, params, false)
    }
}
