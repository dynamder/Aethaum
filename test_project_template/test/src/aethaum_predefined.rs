pub trait Describe {
    fn describe(&self) -> &'static str {
        ""
    }
    fn describe_field(&self, field_name: &str) -> &'static str {
        ""
    }
}
#[derive(Event)]
pub struct AethaumSpawnEntity {
    pub prototype_name: String,
    pub entity_response: Option<oneshot::Sender<Entity>>,
}
