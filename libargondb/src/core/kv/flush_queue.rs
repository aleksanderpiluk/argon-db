#[derive(Debug, Clone)]
enum MemtableFlushQueue {
    Active {
        sender: Sender<KVMemtableFlushRequest>,
        receiver: Receiver<KVMemtableFlushRequest>,
    },
    Closed {
        receiver: Receiver<KVMemtableFlushRequest>,
    },
}

impl MemtableFlushQueue {
    fn new() -> Self {
        let (sender, receiver) = flume::unbounded();

        Self::Active { sender, receiver }
    }

    fn close(&mut self) {
        let Self::Active {
            sender: _,
            receiver,
        } = self
        else {
            println!("memtable flush queue already closed");
            return;
        };

        *self = Self::Closed {
            receiver: receiver.clone(),
        };
    }

    fn receiver(&self) -> &Receiver<KVMemtableFlushRequest> {
        match self {
            Self::Active {
                sender: _,
                receiver,
            } => receiver,
            Self::Closed { receiver } => receiver,
        }
    }

    fn sender(&self) -> Option<&Sender<KVMemtableFlushRequest>> {
        match self {
            Self::Active {
                sender,
                receiver: _,
            } => Some(sender),
            Self::Closed { receiver: _ } => None,
        }
    }
}

pub struct KVMemtableFlushRequest {
    pub memtable: Arc<Memtable>,
}
