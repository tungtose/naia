use byteorder::{BigEndian, ReadBytesExt};
use std::collections::VecDeque;

use naia_shared::{EntityType, EventType, LocalEntityKey, Manifest, PacketReader, SequenceBuffer};

const COMMAND_BUFFER_MAX_SIZE: u16 = 64;

/// Handles incoming commands, buffering them to be received on the correct tick
#[derive(Debug)]
pub struct CommandReceiver<T: EventType> {
    queued_incoming_commands: SequenceBuffer<VecDeque<(u16, T)>>,
}

impl<T: EventType> CommandReceiver<T> {
    /// Creates a new CommandReceiver
    pub fn new() -> Self {
        CommandReceiver {
            queued_incoming_commands: SequenceBuffer::with_capacity(COMMAND_BUFFER_MAX_SIZE),
        }
    }

    /// Get the most recently received Command
    pub fn pop_incoming_command(&mut self, server_tick: u16) -> Option<(LocalEntityKey, T)> {
        if let Some(queue) = self.queued_incoming_commands.get_mut(server_tick) {
            return queue.pop_front();
        }
        return None;
    }

    /// Given incoming packet data, read transmitted Command and store them to
    /// be returned to the application
    pub fn process_data<U: EntityType>(
        &mut self,
        client_tick: u16,
        reader: &mut PacketReader,
        manifest: &Manifest<T, U>,
    ) {
        let command_count = reader.read_u8();
        for _x in 0..command_count {
            let local_entity_key: LocalEntityKey = reader.read_u16();
            let naia_id: u16 = reader.read_u16();
            let payload_length: u8 = reader.read_u8();

            match manifest.create_event(naia_id, reader) {
                Some(new_command) => {
                    if !self.queued_incoming_commands.exists(client_tick) {
                        self.queued_incoming_commands
                            .insert(client_tick, VecDeque::new());
                    }
                    if let Some(queue) = self.queued_incoming_commands.get_mut(client_tick) {
                        queue.push_back((local_entity_key, new_command));
                    }
                }
                _ => {}
            }
        }
    }
}
