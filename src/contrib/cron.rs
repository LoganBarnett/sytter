use std::time::Duration;
use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc::{
    channel,
    Sender,
    TryRecvError::{
        Empty,
        Disconnected,
    }
};

use crate::error::AppError;
use crate::watcher::Watcher;

#[derive(Clone)]
pub struct CronWatch {
    pub cron: String,
    pub shell: String,
    pub watching: bool,
}

impl Watcher for CronWatch {

    fn watch_start(
        &mut self,
        watch_trigger: Box<dyn Fn() + Send + Sync + 'static>,
    ) -> Result<(JoinHandle<()>, Sender<bool>), AppError> {
        self.watching = true;
        let (sender, receiver) = channel();
        let join = thread::spawn(move || {
            let mut watching = true;
            let mut ticks = 0;
            while watching {
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
                match receiver.try_recv() {
                    Ok(b) => watching = b,
                    Err(e) => match e {
                        Empty => (),
                        Disconnected => panic!(),
                    },
                }
                println!("Loop done. Watching? {}", watching);
            }
        });
        Ok((join, sender))
    }

    fn watch_stop(&mut self) -> Result<(), AppError> {
        self.watching = false;
        Ok(())
    }
}
