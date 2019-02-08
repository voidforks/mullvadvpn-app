use std::thread;

use error_chain::ChainedError;
use futures::{
    sync::{mpsc, oneshot},
    Async, Future, Stream,
};
use talpid_types::tunnel::{ActionAfterDisconnect, BlockReason};

use super::{
    BlockedState, ConnectingState, DisconnectedState, EventConsequence, ResultExt,
    SharedTunnelStateValues, TunnelCommand, TunnelState, TunnelStateTransition, TunnelStateWrapper,
};
use crate::tunnel::CloseHandle;

/// This state is active from when we manually trigger a tunnel kill until the tunnel wait
/// operation (TunnelExit) returned.
pub struct DisconnectingState {
    exited: oneshot::Receiver<Option<BlockReason>>,
    after_disconnect: AfterDisconnect,
}

impl DisconnectingState {
    fn handle_commands(
        mut self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        let event = try_handle_event!(self, commands.poll());
        let after_disconnect = self.after_disconnect;

        self.after_disconnect = match after_disconnect {
            AfterDisconnect::Nothing => match event {
                Ok(TunnelCommand::AllowLan(allow_lan)) => {
                    shared_values.allow_lan = allow_lan;
                    AfterDisconnect::Nothing
                }
                Ok(TunnelCommand::BlockWhenDisconnected(block_when_disconnected)) => {
                    shared_values.block_when_disconnected = block_when_disconnected;
                    AfterDisconnect::Nothing
                }
                Ok(TunnelCommand::IsOffline(is_offline)) => {
                    shared_values.is_offline = is_offline;
                    AfterDisconnect::Nothing
                }
                Ok(TunnelCommand::Connect) => AfterDisconnect::Reconnect(0),
                Ok(TunnelCommand::Block(reason)) => AfterDisconnect::Block(reason),
                _ => AfterDisconnect::Nothing,
            },
            AfterDisconnect::Block(reason) => match event {
                Ok(TunnelCommand::AllowLan(allow_lan)) => {
                    shared_values.allow_lan = allow_lan;
                    AfterDisconnect::Block(reason)
                }
                Ok(TunnelCommand::BlockWhenDisconnected(block_when_disconnected)) => {
                    shared_values.block_when_disconnected = block_when_disconnected;
                    AfterDisconnect::Block(reason)
                }
                Ok(TunnelCommand::IsOffline(is_offline)) => {
                    shared_values.is_offline = is_offline;
                    if !is_offline && reason == BlockReason::IsOffline {
                        AfterDisconnect::Reconnect(0)
                    } else {
                        AfterDisconnect::Block(reason)
                    }
                }
                Ok(TunnelCommand::Connect) => AfterDisconnect::Reconnect(0),
                Ok(TunnelCommand::Disconnect) => AfterDisconnect::Nothing,
                Ok(TunnelCommand::Block(new_reason)) => AfterDisconnect::Block(new_reason),
                Err(_) => AfterDisconnect::Block(reason),
            },
            AfterDisconnect::Reconnect(retry_attempt) => match event {
                Ok(TunnelCommand::AllowLan(allow_lan)) => {
                    shared_values.allow_lan = allow_lan;
                    AfterDisconnect::Reconnect(retry_attempt)
                }
                Ok(TunnelCommand::BlockWhenDisconnected(block_when_disconnected)) => {
                    shared_values.block_when_disconnected = block_when_disconnected;
                    AfterDisconnect::Reconnect(retry_attempt)
                }
                Ok(TunnelCommand::IsOffline(is_offline)) => {
                    shared_values.is_offline = is_offline;
                    if is_offline {
                        AfterDisconnect::Block(BlockReason::IsOffline)
                    } else {
                        AfterDisconnect::Reconnect(retry_attempt)
                    }
                }
                Ok(TunnelCommand::Connect) => AfterDisconnect::Reconnect(retry_attempt),
                Ok(TunnelCommand::Disconnect) | Err(_) => AfterDisconnect::Nothing,
                Ok(TunnelCommand::Block(reason)) => AfterDisconnect::Block(reason),
            },
        };

        EventConsequence::SameState(self)
    }

    fn handle_exit_event(
        mut self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match self.exited.poll() {
            Ok(Async::NotReady) => NoEvents(self),
            Ok(Async::Ready(block_reason)) => {
                NewState(self.after_disconnect(block_reason, shared_values))
            }
            Err(_) => NewState(self.after_disconnect(None, shared_values)),
        }
    }

    fn after_disconnect(
        self,
        block_reason: Option<BlockReason>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        if let Some(reason) = block_reason {
            return BlockedState::enter(shared_values, reason);
        }

        match self.after_disconnect {
            AfterDisconnect::Nothing => DisconnectedState::enter(shared_values, ()),
            AfterDisconnect::Block(reason) => BlockedState::enter(shared_values, reason),
            AfterDisconnect::Reconnect(retry_attempt) => {
                ConnectingState::enter(shared_values, retry_attempt)
            }
        }
    }
}

impl TunnelState for DisconnectingState {
    type Bootstrap = (
        CloseHandle,
        oneshot::Receiver<Option<BlockReason>>,
        AfterDisconnect,
    );

    fn enter(
        _: &mut SharedTunnelStateValues,
        (close_handle, exited, after_disconnect): Self::Bootstrap,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        thread::spawn(move || {
            let close_result = close_handle
                .close()
                .chain_err(|| "Failed to close the tunnel");

            if let Err(error) = close_result {
                log::error!("{}", error.display_chain());
            }
        });

        let action_after_disconnect = after_disconnect.action();

        (
            TunnelStateWrapper::from(DisconnectingState {
                exited,
                after_disconnect,
            }),
            TunnelStateTransition::Disconnecting(action_after_disconnect),
        )
    }

    fn handle_event(
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        self.handle_commands(commands, shared_values)
            .or_else(Self::handle_exit_event, shared_values)
    }
}

/// Which state should be transitioned to after disconnection is complete.
pub enum AfterDisconnect {
    Nothing,
    Block(BlockReason),
    Reconnect(u32),
}

impl AfterDisconnect {
    /// Build event representation of the action that will be taken after the disconnection.
    pub fn action(&self) -> ActionAfterDisconnect {
        match self {
            AfterDisconnect::Nothing => ActionAfterDisconnect::Nothing,
            AfterDisconnect::Block(..) => ActionAfterDisconnect::Block,
            AfterDisconnect::Reconnect(..) => ActionAfterDisconnect::Reconnect,
        }
    }
}
