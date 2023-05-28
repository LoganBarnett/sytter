use std::time::Duration;
use std::thread;

use crate::error::AppError;
use crate::sytter::Watcher;

pub struct CronWatch {
    pub cron: String,
    pub shell: String,
    pub watching: bool,
}

impl Watcher for CronWatch {

    fn watch_start(
        &mut self,
        watch_trigger: Box<dyn Fn() + Send + 'static>,
    ) -> Result<(), AppError> {
        self.watching = true;
        thread::spawn(move || {
            let mut ticks = 0;
            while self.watching {
                // Use a low sleep duration so we can shut down Sytter quickly.
                let duration = Duration::from_millis(1000);
                thread::sleep(duration);
                ticks = ticks + 1;
                if ticks % (5 * 60) == 0 {
                    watch_trigger()
                    // f();
                    // if shell_exec_check(&self.shell, &vec!(2))? {
                    // }
                }
            }
        });
        Ok(())
    }

    fn watch_stop(&mut self) -> Result<(), AppError> {
        self.watching = false;
        Ok(())
    }
}
