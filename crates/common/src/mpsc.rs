use tokio::sync::mpsc::{Receiver, Sender};

/// A Multi-Producer, Single-Consumer channel.
///
/// Producers dispatch messages to a single consumer.
pub struct MpSc<M> {
    /// to send messages to other tasks
    pub sender: Sender<M>,
    /// The receiver is never used it is just to keep the sender alive.
    reciever: Option<Receiver<M>>,
}


impl<M> Clone for MpSc<M> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            reciever: None,
        }
    }
}



impl<M> MpSc<M> {
    /// Creates a new MpSc channel with a buffer size.
    ///
    /// This method should be called by the main application thread only,
    /// the mpsc should be passed to other threads by cloning it. With multiple producers
    /// getting used whereas only a single consumer is allowed.
    pub fn new(buffer_size: usize) -> Self {
        let (sender, reciever) = tokio::sync::mpsc::channel(buffer_size);
        Self {
            sender,
            reciever: Some(reciever),
        }
    }


    /// Clones the sender and the receiver.
    ///
    /// This method should be called by the handler thread only (only one consumer).
    pub fn clone_with_receiver(&mut self) -> Self {
        Self {
            sender: self.sender(),
            reciever: self.receiver(),
        }
    }

    /// Get a sender to send messages to other tasks.
    pub fn sender(&self) -> Sender<M> {
        self.sender.clone()
    }

    /// Get a receiver to receive messages from other tasks.
    /// It will replace the existing receiver with None as
    /// only one receiver is allowed.
    pub fn receiver(&mut self) -> Option<Receiver<M>> {
        self.reciever.take()
    }

}
