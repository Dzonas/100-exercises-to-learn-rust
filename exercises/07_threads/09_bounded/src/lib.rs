// TODO: Convert the implementation to use bounded channels.
use crate::data::{Ticket, TicketDraft};
use crate::store::{TicketId, TicketStore};
use std::sync::mpsc::{Receiver, SyncSender};

pub mod data;
pub mod store;

#[derive(Clone)]
pub struct TicketStoreClient {
    capacity: usize,
    sender: SyncSender<Command>
}

#[derive(Debug)]
pub struct TicketStoreClientError;

impl TicketStoreClient {
    pub fn insert(&self, draft: TicketDraft) -> Result<TicketId, TicketStoreClientError> {
	let (sender, receiver) = std::sync::mpsc::sync_channel(self.capacity);
        let command = Command::Insert { draft, response_channel: sender };
	self.sender.try_send(command).map_err(|_| TicketStoreClientError)?;

	Ok(receiver.recv().unwrap())
    }

    pub fn get(&self, id: TicketId) -> Result<Option<Ticket>, TicketStoreClientError> {
	let (sender, receiver) = std::sync::mpsc::sync_channel(self.capacity);
        let command = Command::Get { id, response_channel: sender };
	self.sender.try_send(command).map_err(|_| TicketStoreClientError)?;

	Ok(Some(receiver.recv().unwrap()))
    }
}

pub fn launch(capacity: usize) -> TicketStoreClient {
    let (sender, receiver) = std::sync::mpsc::sync_channel(capacity);
    std::thread::spawn(move || server(receiver));

    TicketStoreClient { capacity, sender }
}

enum Command {
    Insert {
        draft: TicketDraft,
        response_channel: SyncSender<TicketId>
    },
    Get {
        id: TicketId,
        response_channel: SyncSender<Ticket>
    },
}

pub fn server(receiver: Receiver<Command>) {
    let mut store = TicketStore::new();
    loop {
        match receiver.recv() {
            Ok(Command::Insert {
                draft,
                response_channel,
            }) => {
                let id = store.add_ticket(draft);
		response_channel.send(id).unwrap();
            }
            Ok(Command::Get {
                id,
                response_channel,
            }) => {
                let ticket = store.get(id);
		response_channel.send(ticket.unwrap().clone()).unwrap();
            }
            Err(_) => {
                // There are no more senders, so we can safely break
                // and shut down the server.
                break;
            }
        }
    }
}
