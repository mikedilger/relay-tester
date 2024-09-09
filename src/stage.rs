use crate::error::Error;
use crate::globals::GLOBALS;
use std::time::Duration;
use strum_macros::{EnumCount, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumCount, EnumIter)]
#[repr(usize)]
pub enum Stage {
    Preauth,
    Registered,
    Stranger,
    Unknown,
}

impl Stage {
    pub async fn init(&self) -> Result<(), Error> {
        match *self {
            Stage::Preauth => {
                // nothing to setup
            }
            Stage::Registered => {
                GLOBALS
                    .connection
                    .write()
                    .as_mut()
                    .unwrap()
                    .authenticate_registered_if_challenged()
                    .await?;

                // TBD: Inject Event Group A
            }
            Stage::Stranger => {
                GLOBALS
                    .connection
                    .write()
                    .as_mut()
                    .unwrap()
                    .disconnect()
                    .await?;

                GLOBALS
                    .connection
                    .write()
                    .as_mut()
                    .unwrap()
                    .reconnect()
                    .await?;

                let _ = GLOBALS
                    .connection
                    .write()
                    .as_mut()
                    .unwrap()
                    .wait_for_message(Duration::from_secs(1))
                    .await?;

                GLOBALS
                    .connection
                    .write()
                    .as_mut()
                    .unwrap()
                    .authenticate_stranger_if_challenged()
                    .await?;
            }
            Stage::Unknown => {
                // nothing to setup
            }
        }

        Ok(())
    }
}