use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub member_ids: Vec<Uuid>,  // references into AppState.items — must be validated on load (see Phase 1)
    pub bg_color: [u8; 4],
    pub text_color: [u8; 4],
    pub grid_pos: (i32, i32),
}
