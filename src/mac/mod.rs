//! Partial implementation of the IEEE 802.15.4 MAC layer

pub mod beacon;
pub mod command;
pub mod frame;

#[cfg(feature = "security")]
pub use crate::mac::frame::FrameSerDesContext;
pub use frame::header::{
    Address, AddressMode, ExtendedAddress, FrameType, FrameVersion, Header,
    PanId, ShortAddress,
};
pub use frame::{security, DecodeError, FooterMode, Frame, FrameContent};
