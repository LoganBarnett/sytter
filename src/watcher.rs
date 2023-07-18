use crate::{error::AppError};
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

// A watcher should just be a specialty kind of trigger.
pub trait Watcher {

    fn watch_start(
        &mut self,
        watch_trigger: Box<dyn Fn() + Send + Sync + 'static>,
    ) -> Result<(JoinHandle<()>, Sender<bool>), AppError>;

    fn watch_stop(&mut self) -> Result<(), AppError>;
}
