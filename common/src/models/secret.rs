use chrono::{DateTime, Utc};
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS,
    presets::{NOTHING, UTF8_FULL},
    Attribute, Cell, CellAlignment, ContentArrangement, Table,
};
use crossterm::style::Stylize;
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

use crate::secrets::Secret;

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = shuttle_common::models::secret::Response))]
pub struct Response {
    pub key: String,
    pub value: Secret<String>,
    #[cfg_attr(feature = "openapi", schema(value_type = KnownFormat::DateTime))]
    pub last_update: DateTime<Utc>,
}

pub fn get_secrets_table(secrets: &Vec<Response>, raw: bool) -> String {
    if secrets.is_empty() {
        if raw {
            "No secrets are linked to this service\n".to_string()
        } else {
            format!("{}\n", "No secrets are linked to this service".bold())
        }
    } else {
        let mut table = Table::new();

        if raw {
            table
                .load_preset(NOTHING)
                .set_content_arrangement(ContentArrangement::Disabled)
                .set_header(vec![
                    Cell::new("Key").set_alignment(CellAlignment::Left),
                    Cell::new("Last updated").set_alignment(CellAlignment::Left),
                ]);
        } else {
            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_content_arrangement(ContentArrangement::DynamicFullWidth)
                .set_header(vec![
                    Cell::new("Key")
                        .set_alignment(CellAlignment::Center)
                        .add_attribute(Attribute::Bold),
                    Cell::new("Last updated")
                        .set_alignment(CellAlignment::Center)
                        .add_attribute(Attribute::Bold),
                ]);
        }

        for resource in secrets.iter() {
            table.add_row(vec![
                resource.key.to_string(),
                resource
                    .last_update
                    .format("%Y-%m-%dT%H:%M:%SZ")
                    .to_string(),
            ]);
        }

        format!("These secrets are linked to this service\n{table}\n")
    }
}
