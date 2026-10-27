
use async_channel::{Sender, TrySendError};
use warpui::{ModelHandle, ViewHandle};

use crate::terminal::writeable_pty::pty_controller::{EventLoopSendError, EventLoopSender};
use crate::terminal::writeable_pty::Message;
use crate::terminal::{
    TerminalManager as TerminalManagerTrait,
    TerminalView,
};


#[cfg_attr(not(feature = "remote_tty"), allow(dead_code))]
pub struct TerminalManagerInit {
    pub(crate) manager: ModelHandle<Box<dyn TerminalManagerTrait>>,
    pub(crate) view: ViewHandle<TerminalView>,
}



impl EventLoopSender for Sender<Message> {
    fn send(&self, message: Message) -> Result<(), EventLoopSendError> {
        self.try_send(message).map_err(|err| match err {
            TrySendError::Closed(_) => EventLoopSendError::Disconnected,
            TrySendError::Full(_) => EventLoopSendError::Other(err.into()),
        })
    }
}
