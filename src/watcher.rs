use crate::{error::AppError};

// A watcher should just be a specialty kind of trigger.
pub trait Watcher {
    fn watch_start(
        &mut self,
        watch_trigger: Box<dyn Fn() + Send + 'static>,
    ) -> Result<(), AppError>;
    fn watch_stop(&mut self) -> Result<(), AppError>;
}
