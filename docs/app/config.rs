use crate::components::state::{RequestMetrics, SiteMetadata};
use antixt::{Application, StartupError};
use std::sync::Arc;

pub fn configure(application: &mut Application) -> Result<(), StartupError> {
    application.state(SiteMetadata {
        name: "antixt",
        version: "v0.4",
    })?;
    let metrics = Arc::new(RequestMetrics::default());
    application.state(Arc::clone(&metrics))?;
    application.lifecycle(metrics);
    Ok(())
}
