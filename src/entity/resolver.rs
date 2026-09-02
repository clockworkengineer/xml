use crate::error::Result;

pub trait EntityResolver: Send + Sync {
    fn resolve_entity(&self, public_id: Option<&str>, system_id: &str) -> Result<String>;
}

pub struct DefaultEntityResolver;

impl EntityResolver for DefaultEntityResolver {
    fn resolve_entity(&self, _public_id: Option<&str>, _system_id: &str) -> Result<String> {
        Ok(String::new())
    }
}
