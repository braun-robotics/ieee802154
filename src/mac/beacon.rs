//! Beacon
//!
//! Work in progress

use byte::{check_len, BytesExt, TryRead, TryWrite};
use core::convert::From;

use crate::mac::{ExtendedAddress, ShortAddress};

/// Beacon order is used to calculate the beacon interval
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BeaconOrder {
    /// Used to calculate at which interval beacons are sent
    ///
    /// Beacon interval  = BaseSuperframeDuration × (2 ^ BeaconOrder)
    BeaconOrder(u8),
    /// Beacon are only sent on demand
    OnDemand,
}

impl From<u8> for BeaconOrder {
    /// Convert u8 to beacon order
    fn from(value: u8) -> Self {
        match value {
            0..=14 => BeaconOrder::BeaconOrder(value),
            _ => BeaconOrder::OnDemand,
        }
    }
}

impl From<BeaconOrder> for u8 {
    /// Convert beacon order to u8
    fn from(value: BeaconOrder) -> Self {
        match value {
            BeaconOrder::BeaconOrder(v) => v,
            BeaconOrder::OnDemand => 15,
        }
    }
}

/// Superframe order, amount of time during which this superframe is active
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SuperframeOrder {
    /// Amount of time that the superframe is active
    ///
    /// superframe duration = base superframe duration × (2 ^ superframe order)
    SuperframeOrder(u8),
    /// No superframes are sent
    Inactive,
}

impl From<u8> for SuperframeOrder {
    /// Convert u8 to superframe order
    fn from(value: u8) -> Self {
        match value {
            0..=14 => SuperframeOrder::SuperframeOrder(value),
            _ => SuperframeOrder::Inactive,
        }
    }
}

impl From<SuperframeOrder> for u8 {
    /// Convert superframe order to u8
    fn from(value: SuperframeOrder) -> Self {
        match value {
            SuperframeOrder::SuperframeOrder(v) => v,
            SuperframeOrder::Inactive => 15,
        }
    }
}

/// Superframe specification
///
/// The superframe specification describes the organisation of frames in the
/// air when using superframes and/or periodical beacons.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SuperframeSpecification {
    /// Beacon order, 0-15, where 15 is on demand.
    ///
    /// Beacon interval  = BaseSuperframeDuration × (2 ^ BeaconOrder)
    pub beacon_order: BeaconOrder,
    /// Superframe order, amount of time during which this superframe is active
    pub superframe_order: SuperframeOrder,
    /// final contention access period slot used
    pub final_cap_slot: u8,
    /// Limit receiving of beacons for a period. Not used if beacon_order is OnDemand.
    pub battery_life_extension: bool,
    /// Frame sent by a coordinator
    pub pan_coordinator: bool,
    /// The coordinator accepts associations to the PAN
    pub association_permit: bool,
}

const BATTERY_LIFE_EXTENSION: u8 = 0b0001_0000;
const PAN_COORDINATOR: u8 = 0b0100_0000;
const ASSOCIATION_PERMIT: u8 = 0b1000_0000;

impl TryRead<'_> for SuperframeSpecification {
    fn try_read(bytes: &[u8], _ctx: ()) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        check_len(&bytes, 2)?;
        let byte: u8 = bytes.read(offset)?;
        let beacon_order = BeaconOrder::from(byte & 0x0f);
        let superframe_order = SuperframeOrder::from((byte >> 4) & 0x0f);
        let byte: u8 = bytes.read(offset)?;
        let final_cap_slot = byte & 0x0f;
        let battery_life_extension =
            (byte & BATTERY_LIFE_EXTENSION) == BATTERY_LIFE_EXTENSION;
        let pan_coordinator = (byte & PAN_COORDINATOR) == PAN_COORDINATOR;
        let association_permit =
            (byte & ASSOCIATION_PERMIT) == ASSOCIATION_PERMIT;
        Ok((
            Self {
                beacon_order,
                superframe_order,
                final_cap_slot,
                battery_life_extension,
                pan_coordinator,
                association_permit,
            },
            *offset,
        ))
    }
}

impl TryWrite for SuperframeSpecification {
    fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
        let offset = &mut 0;
        let bo = u8::from(self.beacon_order.clone());
        let so = u8::from(self.superframe_order.clone());
        bytes.write(offset, (bo & 0x0f) | (so << 4))?;
        let ble = if self.battery_life_extension {
            BATTERY_LIFE_EXTENSION
        } else {
            0
        };
        let pc = if self.pan_coordinator {
            PAN_COORDINATOR
        } else {
            0
        };
        let ap = if self.association_permit {
            ASSOCIATION_PERMIT
        } else {
            0
        };
        bytes.write(offset, self.final_cap_slot & 0x0f | ble | pc | ap)?;
        Ok(*offset)
    }
}

/// Direction of data
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Direction {
    /// Receive data
    Receive,
    /// Transmit data
    Transmit,
}

/// Descriptor of the guaranteed time slots (GTSs)
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GuaranteedTimeSlotDescriptor {
    /// Device short address used by this slot
    pub short_address: ShortAddress,
    /// Slot start
    pub starting_slot: u8,
    /// Slot length
    pub length: u8,
    /// Direction of the slot, either transmit or receive
    pub direction: Direction,
}

impl GuaranteedTimeSlotDescriptor {
    /// Create a new empty slot
    pub fn new() -> Self {
        GuaranteedTimeSlotDescriptor {
            short_address: ShortAddress::broadcast(),
            starting_slot: 0,
            length: 0,
            direction: Direction::Receive,
        }
    }
}

impl TryRead<'_> for GuaranteedTimeSlotDescriptor {
    fn try_read(bytes: &[u8], _ctx: ()) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        check_len(&bytes, 3)?;
        let short_address = bytes.read(offset)?;
        let byte: u8 = bytes.read(offset)?;
        let starting_slot = byte & 0x0f;
        let length = byte >> 4;
        Ok((
            Self {
                short_address,
                starting_slot,
                length,
                // This should be updated by the super
                direction: Direction::Receive,
            },
            *offset,
        ))
    }
}

impl TryWrite for GuaranteedTimeSlotDescriptor {
    fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
        let offset = &mut 0;
        bytes.write(offset, self.short_address)?;
        bytes.write(offset, self.starting_slot | self.length << 4)?;
        Ok(*offset)
    }
}

impl GuaranteedTimeSlotDescriptor {
    /// Set the direction for this slot
    fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }
    /// Returns `true` if this is a transmit slot
    fn direction_transmit(&self) -> bool {
        self.direction == Direction::Transmit
    }
}

const COUNT_MASK: u8 = 0b0000_0111;
const PERMIT: u8 = 0b1000_0000;

/// Information of the guaranteed time slots (GTSs)
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GuaranteedTimeSlotInformation {
    /// Permit GTS
    pub permit: bool,
    /// Time slot information
    pub slots: heapless::Vec<GuaranteedTimeSlotDescriptor, 7>,
}

impl GuaranteedTimeSlotInformation {
    /// Create a new empty GTS information
    pub fn new() -> Self {
        GuaranteedTimeSlotInformation {
            permit: false,
            slots: heapless::Vec::new(),
        }
    }

    /// Get the slots as a slice
    pub fn slots(&self) -> &[GuaranteedTimeSlotDescriptor] {
        self.slots.as_slice()
    }
}

impl TryWrite for GuaranteedTimeSlotInformation {
    fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
        let offset = &mut 0;
        assert!(self.slots.capacity() <= 7);
        let permit = if self.permit { PERMIT } else { 0 };

        let header = ((self.slots.len() as u8) & COUNT_MASK) | permit;
        bytes.write(offset, header)?;

        if !self.slots.is_empty() {
            let direction_mask = {
                let mut dir = 0x01;
                let mut direction_mask = 0u8;
                for slot in &self.slots {
                    if slot.direction_transmit() {
                        direction_mask = direction_mask | dir;
                    }
                    dir = dir << 1;
                }
                direction_mask
            };

            bytes.write(offset, direction_mask)?;

            for slot in self.slots {
                bytes.write(offset, slot)?;
            }
        }
        Ok(*offset)
    }
}

impl TryRead<'_> for GuaranteedTimeSlotInformation {
    fn try_read(bytes: &[u8], _ctx: ()) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let byte: u8 = bytes.read(offset)?;
        let slot_count = (byte & COUNT_MASK) as usize;
        let permit = (byte & PERMIT) == PERMIT;
        let mut slots = heapless::Vec::new();
        assert!(slot_count <= slots.capacity());
        if slot_count > 0 {
            check_len(&bytes[*offset..], 2 + (3 * slot_count))?;
            let mut direction_mask: u8 = bytes.read(offset)?;
            for _ in 0..slot_count {
                let mut slot: GuaranteedTimeSlotDescriptor =
                    bytes.read(offset)?;
                let direction = if direction_mask & 0b1 == 0b1 {
                    Direction::Transmit
                } else {
                    Direction::Receive
                };
                slot.set_direction(direction);
                direction_mask = direction_mask >> 1;
                slots.push(slot).expect(
                    "slot_count can never be larger than Vec::capacity",
                );
            }
        }
        Ok((Self { permit, slots }, *offset))
    }
}

const SHORT_MASK: u8 = 0b0000_0111;
const EXTENDED_MASK: u8 = 0b0111_0000;

/// # Pending Address(es)
///
/// Addresses to devices that has pending messages with the coordinator
///
/// ```txt
/// +--------+-----------------+--------------------+
/// | Header | Short Addresses | Extended Addresses |
/// +--------+-----------------+--------------------+
///     1          0 - 14             0 - 448          octets
/// ```
///
/// ## Header
///
/// ```txt
/// +-------------+----------+----------------+----------+
/// | Short Count | Reserved | Extended Count | Reserved |
/// +-------------+----------+----------------+----------+
///      0 - 2         3         4 - 6             7        bit
/// ```
#[derive(Clone, Default, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PendingAddress {
    /// List of pending [`ShortAddress`]es
    pub short_addresses: heapless::Vec<ShortAddress, 7>,
    /// List of pending [`ExtendedAddress`]es
    pub extended_addresses: heapless::Vec<ExtendedAddress, 7>,
}

impl PendingAddress {
    /// Create a new empty PendingAddress struct
    pub const fn new() -> Self {
        Self {
            short_addresses: heapless::Vec::new(),
            extended_addresses: heapless::Vec::new(),
        }
    }

    /// Get the short addresses
    pub fn short_addresses(&self) -> &[ShortAddress] {
        self.short_addresses.as_slice()
    }
    /// Get the extended address
    pub fn extended_addresses(&self) -> &[ExtendedAddress] {
        self.extended_addresses.as_slice()
    }
}

impl TryRead<'_> for PendingAddress {
    fn try_read(bytes: &[u8], _ctx: ()) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let ss = size_of::<ShortAddress>();
        let es = size_of::<ExtendedAddress>();
        let byte: u8 = bytes.read(offset)?;
        let sl = (byte & SHORT_MASK) as usize;
        let el = ((byte & EXTENDED_MASK) >> 4) as usize;
        check_len(&bytes[*offset..], (sl * ss) + (el * es))?;
        let mut short_addresses = heapless::Vec::new();
        assert!(sl <= short_addresses.capacity());
        for _ in 0..sl {
            short_addresses
                .push(bytes.read(offset)?)
                .expect("sl can never be larger than 7");
        }
        let mut extended_addresses = heapless::Vec::new();
        assert!(el <= extended_addresses.capacity());
        for _ in 0..el {
            extended_addresses
                .push(bytes.read(offset)?)
                .expect("el can never be larger than 7");
        }
        Ok((
            Self {
                short_addresses,
                extended_addresses,
            },
            *offset,
        ))
    }
}

impl TryWrite for PendingAddress {
    fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
        let offset = &mut 0;
        assert!(self.short_addresses.capacity() <= 7);
        assert!(self.extended_addresses.capacity() <= 7);

        let sl = self.short_addresses.len();
        let el = self.extended_addresses.len();

        // Combine list lengths into one field, see Table 45 of IEEE 802.15.4-2011
        let combined_lengths =
            (((el as u8) << 4) & EXTENDED_MASK) | ((sl as u8) & SHORT_MASK);
        bytes.write(offset, combined_lengths)?;

        for addr in self.short_addresses {
            bytes.write(offset, addr)?;
        }

        for addr in self.extended_addresses {
            bytes.write(offset, addr)?;
        }
        Ok(*offset)
    }
}

/// Beacon frame
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Beacon {
    /// Superframe specification
    pub superframe_spec: SuperframeSpecification,
    /// Guaranteed time slot information
    pub guaranteed_time_slot_info: GuaranteedTimeSlotInformation,
    /// Pending addresses
    pub pending_address: PendingAddress,
}

impl TryRead<'_> for Beacon {
    fn try_read(bytes: &[u8], _ctx: ()) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        Ok((
            Self {
                superframe_spec: bytes.read(offset)?,
                guaranteed_time_slot_info: bytes.read(offset)?,
                pending_address: bytes.read(offset)?,
            },
            *offset,
        ))
    }
}

impl TryWrite for Beacon {
    fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
        let offset = &mut 0;
        bytes.write(offset, self.superframe_spec)?;
        bytes.write(offset, self.guaranteed_time_slot_info)?;
        bytes.write(offset, self.pending_address)?;
        Ok(*offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use heapless::Vec;

    #[test]
    fn decode_superframe_specification() {
        let data = [0xff, 0x0f];
        let mut len = 0usize;
        let ss: SuperframeSpecification = data.read(&mut len).unwrap();
        assert_eq!(len, data.len());

        assert_eq!(ss.beacon_order, BeaconOrder::OnDemand);
        assert_eq!(ss.superframe_order, SuperframeOrder::Inactive);
        assert_eq!(ss.final_cap_slot, 0xf);
        assert_eq!(ss.battery_life_extension, false);
        assert_eq!(ss.pan_coordinator, false);
        assert_eq!(ss.association_permit, false);
    }

    #[test]
    fn decode_gts_information() {
        let data = [0x00];
        let mut len = 0usize;
        let gts: GuaranteedTimeSlotInformation = data.read(&mut len).unwrap();
        assert_eq!(len, data.len());
        assert_eq!(gts.permit, false);
        assert_eq!(gts.slots().len(), 0);
    }

    #[test]
    fn decode_pending_address() {
        let data = [0x00];
        let mut len = 0usize;
        let pa: PendingAddress = data.read(&mut len).unwrap();
        assert_eq!(len, data.len());
        assert_eq!(pa.short_addresses().len(), 0);
        assert_eq!(pa.extended_addresses().len(), 0);
    }

    #[test]
    fn decode_beacon() {
        let data = [0xff, 0x0f, 0x00, 0x00];
        let mut len = 0usize;
        let beacon: Beacon = data.read(&mut len).unwrap();
        assert_eq!(len, data.len());

        assert_eq!(beacon.superframe_spec.beacon_order, BeaconOrder::OnDemand);
        assert_eq!(
            beacon.superframe_spec.superframe_order,
            SuperframeOrder::Inactive
        );
        assert_eq!(beacon.superframe_spec.final_cap_slot, 0xf);
        assert_eq!(beacon.superframe_spec.battery_life_extension, false);
        assert_eq!(beacon.superframe_spec.pan_coordinator, false);
        assert_eq!(beacon.superframe_spec.association_permit, false);

        assert_eq!(beacon.guaranteed_time_slot_info.permit, false);
        assert_eq!(beacon.guaranteed_time_slot_info.slots().len(), 0);

        assert_eq!(beacon.pending_address.short_addresses().len(), 0);
        assert_eq!(beacon.pending_address.extended_addresses().len(), 0);

        let data = &[
            0x12, 0xc3, 0x82, 0x01, 0x34, 0x12, 0x11, 0x78, 0x56, 0x14, 0x00,
        ][..];
        let mut len = 0usize;
        let beacon: Beacon = data.read(&mut len).unwrap();
        assert_eq!(len, data.len());

        assert_eq!(
            beacon.superframe_spec.beacon_order,
            BeaconOrder::BeaconOrder(2)
        );
        assert_eq!(
            beacon.superframe_spec.superframe_order,
            SuperframeOrder::SuperframeOrder(1)
        );
        assert_eq!(beacon.superframe_spec.final_cap_slot, 3);
        assert_eq!(beacon.superframe_spec.battery_life_extension, false);
        assert_eq!(beacon.superframe_spec.pan_coordinator, true);
        assert_eq!(beacon.superframe_spec.association_permit, true);

        assert_eq!(beacon.guaranteed_time_slot_info.permit, true);
        let slots = beacon.guaranteed_time_slot_info.slots();
        assert_eq!(slots.len(), 2);
        assert_eq!(slots[0].short_address, ShortAddress(0x1234));
        assert_eq!(slots[0].starting_slot, 1);
        assert_eq!(slots[0].length, 1);
        assert_eq!(slots[0].direction, Direction::Transmit);
        assert_eq!(slots[1].short_address, ShortAddress(0x5678));
        assert_eq!(slots[1].starting_slot, 4);
        assert_eq!(slots[1].length, 1);
        assert_eq!(slots[1].direction, Direction::Receive);

        assert_eq!(beacon.pending_address.short_addresses().len(), 0);
        assert_eq!(beacon.pending_address.extended_addresses().len(), 0);

        let data = &[
            0x12, 0xc3, 0x82, 0x02, 0x34, 0x12, 0x11, 0x78, 0x56, 0x14, 0x12,
            0x34, 0x12, 0x78, 0x56, 0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23,
            0x01,
        ][..];
        let mut len = 0usize;
        let beacon: Beacon = data.read(&mut len).unwrap();
        assert_eq!(len, data.len());

        assert_eq!(
            beacon.superframe_spec.beacon_order,
            BeaconOrder::BeaconOrder(2)
        );
        assert_eq!(
            beacon.superframe_spec.superframe_order,
            SuperframeOrder::SuperframeOrder(1)
        );
        assert_eq!(beacon.superframe_spec.final_cap_slot, 3);
        assert_eq!(beacon.superframe_spec.battery_life_extension, false);
        assert_eq!(beacon.superframe_spec.pan_coordinator, true);
        assert_eq!(beacon.superframe_spec.association_permit, true);

        assert_eq!(beacon.guaranteed_time_slot_info.permit, true);
        let slots = beacon.guaranteed_time_slot_info.slots();
        assert_eq!(slots.len(), 2);
        assert_eq!(slots[0].short_address, ShortAddress(0x1234));
        assert_eq!(slots[0].starting_slot, 1);
        assert_eq!(slots[0].length, 1);
        assert_eq!(slots[0].direction, Direction::Receive);
        assert_eq!(slots[1].short_address, ShortAddress(0x5678));
        assert_eq!(slots[1].starting_slot, 4);
        assert_eq!(slots[1].length, 1);
        assert_eq!(slots[1].direction, Direction::Transmit);

        assert_eq!(beacon.pending_address.short_addresses().len(), 2);
        assert_eq!(
            beacon.pending_address.short_addresses()[0],
            ShortAddress(0x1234)
        );
        assert_eq!(
            beacon.pending_address.short_addresses()[1],
            ShortAddress(0x5678)
        );
        assert_eq!(beacon.pending_address.extended_addresses().len(), 1);
        assert_eq!(
            beacon.pending_address.extended_addresses()[0],
            ExtendedAddress(0x0123456789abcdef)
        );
    }

    #[test]
    fn encode_beacon() {
        let superframe_spec = SuperframeSpecification {
            beacon_order: BeaconOrder::OnDemand,
            superframe_order: SuperframeOrder::Inactive,
            final_cap_slot: 0,
            battery_life_extension: false,
            pan_coordinator: true,
            association_permit: true,
        };

        let slots = Vec::from_slice(&[GuaranteedTimeSlotDescriptor {
            short_address: ShortAddress(0x1234),
            starting_slot: 1,
            length: 1,
            direction: Direction::Transmit,
        }])
        .unwrap();

        let guaranteed_time_slot_info = GuaranteedTimeSlotInformation {
            permit: true,
            slots,
        };

        let short_addresses = Vec::from_slice(&[ShortAddress(0x7856)]).unwrap();
        let extended_addresses =
            Vec::from_slice(&[ExtendedAddress(0xaec24a1c2116e260)]).unwrap();

        let pending_address = PendingAddress {
            short_addresses,
            extended_addresses,
        };

        let beacon = Beacon {
            superframe_spec,
            guaranteed_time_slot_info,
            pending_address,
        };

        let mut buffer = [0u8; 128];
        let mut len = 0usize;
        buffer.write(&mut len, beacon).unwrap();
        assert_eq!(len, 18);
        assert_eq!(
            buffer[..len],
            [
                0xff, 0xc0, 0x81, 0x01, 0x34, 0x12, 0x11, 0x11, 0x56, 0x78,
                0x60, 0xe2, 0x16, 0x21, 0x1c, 0x4a, 0xc2, 0xae
            ]
        );
    }
}
