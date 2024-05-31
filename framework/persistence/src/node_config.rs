use crate::{PersistentStorage, Storage};
use delegate::delegate;
use embedded_storage::Storage as StorageDriver;
use vlcb_core::can::{VlcbCanId, CANID_SIZE};
use vlcb_core::cbus::{EventId, VlcbNodeNumber, EVENT_SIZE, NODENUM_SIZE};
use vlcb_core::module::NodeFlags;
use vlcb_defs::VlcbModeParams;
use core::cell::{RefCell};
use core::mem::MaybeUninit;
use heapless::{FnvIndexMap, Vec};
use rclite::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Error indicating that storage has reached its limit
    Exhausted,
    OutOfRange,
    OccupiedEntry,
}

pub trait NodeConfig {
    type Event: LearnedEvent;
    const MAX_EVENTS: u8;
    const EVENT_VAR_COUNT: u8;
    const NODE_VAR_COUNT: u8;

    fn stored_event_count(&self) -> u8;
    /// Saves the current event in the data store.
    fn save_event(&mut self, evt: &EventId, evs: &[u8]) -> Result<(), Error>;

    fn has_event_with_index(&self, index: u8) -> bool;
    fn restore_event(&mut self, evt: EventId, data: Self::Event) -> Result<(), Error>;
    fn restore_event_unchecked(&mut self, evt: EventId, data: Self::Event) -> Result<(), Error>;

    /// Deletes the current event in the object.
    fn delete_event(&mut self, evt: &EventId);
    fn get_event(&self, evt: &EventId) -> Option<&Self::Event>;
    fn has_event(&self, evt: &EventId) -> bool;
    /// NVs are indexed from 1
    fn get_nv(&self, index: u8) -> Result<u8, Error>;
    fn set_nv(&mut self, index: u8, value: u8) -> Result<(), Error>;
    fn can_id(&self) -> &VlcbCanId;
    fn set_can_id(&mut self, can_id: VlcbCanId);
    fn mode(&self) -> VlcbModeParams;
    fn set_mode_uninitialized(&mut self);
    fn set_mode_normal(&mut self, node_num: VlcbNodeNumber);
    fn node_number(&self) -> &VlcbNodeNumber;
    fn set_node_number(&mut self, node_num: VlcbNodeNumber);
    fn was_reset(&self) -> bool;
    fn raise_reset_flag(&mut self);
    fn clear_reset_flag(&mut self);
    fn set_heartbeat(&mut self, state: bool);
    fn set_event_ack(&mut self, state: bool);
    fn is_heartbeat_on(&self) -> bool;
    fn is_event_ack_on(&self) -> bool;
    fn flags(&self) -> NodeFlags;
    fn set_flags(&mut self, flags: NodeFlags);
}

pub trait LearnedEvent {
    fn new(index: u8, vars: &[u8])-> Self;
    fn index(&self) -> u8;
    fn vars(&self) -> &[u8];
}

pub struct HeaplessLearnedEvent<const EVENT_VAR_COUNT: usize> {
    index: u8,
    vars: Vec<u8, EVENT_VAR_COUNT>
}

impl<const EVENT_VAR_COUNT: usize> LearnedEvent for HeaplessLearnedEvent<EVENT_VAR_COUNT> {
    fn index(&self) -> u8 {
        self.index
    }

    fn vars(&self) -> &[u8] {
        &self.vars
    }

    fn new(index: u8, vars: &[u8]) -> Self {
        Self {
            index,
            vars: Vec::from_slice(vars).unwrap(),
        }
    }
}

pub struct NodeConfigStorage<
    const MAX_EVENTS: usize,
    const EVENT_VAR_COUNT: usize,
    const NODE_VAR_COUNT: usize,
> {
    flags: NodeFlags,
    current_mode: VlcbModeParams,
    can_id: VlcbCanId,
    node_number: VlcbNodeNumber,
    nvs: [u8; NODE_VAR_COUNT],
    events: FnvIndexMap<EventId, HeaplessLearnedEvent<EVENT_VAR_COUNT>, MAX_EVENTS>,
    reset_flag: bool,
}

impl<
    const MAX_EVENTS: usize,
    const EVENT_VAR_COUNT: usize,
    const NODE_VAR_COUNT: usize,
> Default for NodeConfigStorage<MAX_EVENTS, EVENT_VAR_COUNT, NODE_VAR_COUNT> {
    fn default() -> Self {
        Self {
            flags: NodeFlags::empty(),
            current_mode: VlcbModeParams::UNINITIALISED,
            nvs: [UNINITIALISED_VALUE; NODE_VAR_COUNT],
            can_id: VlcbCanId::default(),
            node_number: VlcbNodeNumber::default(),
            events: FnvIndexMap::new(),
            reset_flag: false,
        }
    }
}

impl<
    const MAX_EVENTS: usize,
    const EVENT_VAR_COUNT: usize,
    const NODE_VAR_COUNT: usize,
> NodeConfigStorage<MAX_EVENTS, EVENT_VAR_COUNT, NODE_VAR_COUNT> {
    fn set_event_item(&mut self, event_id: EventId, item: HeaplessLearnedEvent<EVENT_VAR_COUNT>) {
        self.events[&event_id] = item
    }

    fn find_free_event_slot(&self) -> Option<u8> {
        // The map is full, no need to evaluate
        if self.events.len() == MAX_EVENTS {
            return None;
        }
        // First index is 0
        let mut i = 0;

        // The map is empty, no need to evaluate
        if self.events.is_empty() {
            return Some(i);
        }

        // Loop over all indices and try to find them in an array
        while self.events.values().any(|v| v.index == i) {
            i += 1;
        }
        Some(i)
    }
}

impl<
    const MAX_EVENTS: usize,
    const EVENT_VAR_COUNT: usize,
    const NODE_VAR_COUNT: usize,
> Storage for NodeConfigStorage<MAX_EVENTS, EVENT_VAR_COUNT, NODE_VAR_COUNT> {
    fn wipe(&mut self) {
        self.events.clear();
        self.nvs.iter_mut().for_each(|v| *v = 0);
        self.can_id = VlcbCanId::default();
        self.node_number = VlcbNodeNumber::default();
        self.current_mode = VlcbModeParams::UNINITIALISED;
        self.flags = NodeFlags::empty();
        self.reset_flag = true;
    }
}

impl<
    const MAX_EVENTS: usize,
    const EVENT_VAR_COUNT: usize,
    const NODE_VAR_COUNT: usize,
> NodeConfig for NodeConfigStorage<MAX_EVENTS, EVENT_VAR_COUNT, NODE_VAR_COUNT> {
    type Event = HeaplessLearnedEvent<EVENT_VAR_COUNT>;
    const MAX_EVENTS: u8 = MAX_EVENTS as u8;

    const EVENT_VAR_COUNT: u8 = EVENT_VAR_COUNT as u8;

    const NODE_VAR_COUNT: u8 = NODE_VAR_COUNT as u8;

    fn stored_event_count(&self) -> u8 {
        self.events.len() as u8
    }

    fn save_event(&mut self, evt: &EventId, evs: &[u8]) -> Result<(), Error> {
        if let Some(item) = self.events.get_mut(evt) {
            item.vars.copy_from_slice(evs);
            return Ok(());
        }
        if let Some(i) = self.find_free_event_slot() {
            self.events[evt] = HeaplessLearnedEvent{ index: i, vars: Vec::from_slice(&evs).unwrap() };
            return Ok(());
        }
        Err(Error::Exhausted)
    }

    fn delete_event(&mut self, evt: &EventId) {
        self.events.remove(evt);
    }

    fn get_event(&self, evt: &EventId) -> Option<&Self::Event> {
        self.events.get(evt)
    }

    fn has_event(&self, evt: &EventId) -> bool {
        self.events.contains_key(evt)
    }

    fn get_nv(&self, index: u8) -> Result<u8, Error> {
        self.nvs.get(index as usize).copied()
            .ok_or(Error::OutOfRange)
    }

    fn set_nv(&mut self, index: u8, value: u8) -> Result<(), Error> {
        self.nvs.get_mut(index as usize)
            .map(|nv| {
                *nv = value;
            })
            .ok_or(Error::OutOfRange)
    }

    fn can_id(&self) -> &VlcbCanId {
        &self.can_id
    }

    fn set_can_id(&mut self, can_id: VlcbCanId) {
        self.can_id = can_id
    }

    fn mode(&self) -> VlcbModeParams {
        self.current_mode
    }

    fn set_mode_uninitialized(&mut self) {
        self.current_mode = VlcbModeParams::UNINITIALISED;
        self.node_number = VlcbNodeNumber::default();
    }

    fn set_mode_normal(&mut self, node_num: VlcbNodeNumber) {
        self.current_mode = VlcbModeParams::NORMAL;
        self.node_number = node_num;
    }

    fn node_number(&self) -> &VlcbNodeNumber {
        &self.node_number
    }

    fn set_node_number(&mut self, node_num: VlcbNodeNumber) {
        self.node_number = node_num;
    }

    fn was_reset(&self) -> bool {
        self.reset_flag
    }

    fn raise_reset_flag(&mut self) {
        self.reset_flag = true;
    }

    fn clear_reset_flag(&mut self) {
        self.reset_flag = false;
    }

    fn set_heartbeat(&mut self, state: bool) {
        match state {
            true => self.flags.insert(NodeFlags::Heartbeat),
            false => self.flags.remove(NodeFlags::Heartbeat)
        }
    }

    fn set_event_ack(&mut self, state: bool) {
        match state {
            true => self.flags.insert(NodeFlags::EventAck),
            false => self.flags.remove(NodeFlags::EventAck)
        }
    }

    fn is_heartbeat_on(&self) -> bool {
        self.flags.contains(NodeFlags::Heartbeat)
    }

    fn is_event_ack_on(&self) -> bool {
        self.flags.contains(NodeFlags::EventAck)
    }

    fn flags(&self) -> NodeFlags {
        self.flags
    }

    fn set_flags(&mut self, flags: NodeFlags) {
        self.flags = flags
    }

    fn restore_event_unchecked(&mut self, evt: EventId, data: Self::Event) -> Result<(), Error> {
        self.events.insert(evt, data)
            .map(|_|())
            .map_err(|_| Error::Exhausted)
    }

    fn has_event_with_index(&self, index: u8) -> bool {
        self.events.values().any(|e| e.index == index)
    }

    fn restore_event(&mut self, evt: EventId, data: Self::Event) -> Result<(), Error> {
        if self.has_event_with_index(data.index) {
            return Err(Error::OccupiedEntry);
        }
        self.restore_event_unchecked(evt, data)
    }
}

/// Helper function for computing bytes per event generic parameter
pub const fn bytes_per_event(event_var_count: usize) -> usize {
    EVENT_SIZE + event_var_count
}


/// Helper function for picking up readout buffer size
///
/// credit: https://stackoverflow.com/a/53646925
const fn cmax(a: usize, b: usize) -> usize {
    [a, b][(a < b) as usize]
}

const UNINITIALISED_VALUE: u8 = 0xff;
const PERSISTENT_BLOCK_SIZE: u8 = 10;
const FLAGGED_AS_RESET: u8 = 99;
const RESET_FLAG_CLEARED: u8 = 0;

pub struct PersistentNodeConfigStorage<
    D: StorageDriver,
    const OFFSET: usize,
    const MAX_EVENTS: usize,
    const EVENT_VAR_COUNT: usize,
    const BYTES_PER_EVENT: usize,
    const NODE_VAR_COUNT: usize,
> {
    driver: Rc<RefCell<D>>,
    dirty: bool,
    inner: NodeConfigStorage<MAX_EVENTS, EVENT_VAR_COUNT, NODE_VAR_COUNT>,
}

//TODO: handle errors returned by storage driver

impl<
        D: StorageDriver,
        const OFFSET: usize,
        const MAX_EVENTS: usize,
        const EVENT_VAR_COUNT: usize,
        const BYTES_PER_EVENT: usize,
        const NODE_VAR_COUNT: usize,
    > PersistentNodeConfigStorage<D, OFFSET, MAX_EVENTS, EVENT_VAR_COUNT, BYTES_PER_EVENT, NODE_VAR_COUNT>
{
    pub fn new(driver: Rc<RefCell<D>>) -> Self {
        Self {
            driver,
            dirty: false,
            inner: NodeConfigStorage::default(),
        }
    }

    const fn bytes_per_event() -> usize {
        // rust doesn't support generic const expressions yet so this is a workaround by having user to pass the value
        // otherwise calculated in this function. The assert serves as an sanity check.
        // TODO: fix this as soon as possible and change the API
        let expected = EVENT_VAR_COUNT + EVENT_SIZE;
        debug_assert!(BYTES_PER_EVENT == expected, "Generic parameter BYTES_PER_EVENT is different from the expected value (result of EVENT_SIZE + EVENT_VAR_COUNT)");
        expected
    }

    const fn mode_addr() -> usize {
        OFFSET
    }

    const fn can_id_addr() -> usize {
        Self::mode_addr() + 1
    }

    const fn node_num_addr_start() -> usize {
        Self::can_id_addr() + CANID_SIZE
    }

    const fn node_num_addr_end() -> usize {
        Self::node_num_addr_start() + NODENUM_SIZE - 1
    }

    const fn flags_addr() -> usize {
        Self::node_num_addr_end() + 1
    }

    const fn reset_flag_addr() -> usize {
        Self::flags_addr() + 1
    }

    /// Ten bytes from the start left for persistence over multiple resets
    const fn persistent_sub_block_end() -> usize {
        OFFSET + PERSISTENT_BLOCK_SIZE as usize - 1
    }

    const fn event_addr_start() -> usize {
        Self::persistent_sub_block_end() + 1
    }

    const fn event_addr_end() -> usize {
        Self::event_addr_start() + (Self::bytes_per_event() * MAX_EVENTS)
    }

    const fn nv_addr_start() -> usize {
        Self::event_addr_end() + 1
    }

    const fn nv_addr_end() -> usize {
        Self::nv_addr_start() + NODE_VAR_COUNT
    }

    pub const fn block_end() -> usize {
        Self::nv_addr_end()
    }

    /// Reloads the event hash table from persistent memory
    fn reload_event_hash_table(&mut self) {
        // this works only for storages like flash or EEPROM
        // if we are to support other storage types we should
        // add more flexible API support, preferably out of scope
        // of this implementation and into a separate reader abstraction
        const UNUSED_ENTRY: [u8; EVENT_SIZE] = [UNINITIALISED_VALUE; EVENT_SIZE];

        // SAFETY: get block of memory for readout, we don't care about initializing it
        #[allow(unsafe_code, clippy::uninit_assumed_init)]
        let mut buf = unsafe {[const { MaybeUninit::<u8>::uninit().assume_init() }; BYTES_PER_EVENT]};

        let mut storage = self.driver.borrow_mut();
        for (index, addr) in (Self::event_addr_start()..=Self::event_addr_end())
            .step_by(Self::bytes_per_event())
            .enumerate()
        {

            let _ = storage.read(addr as u32, &mut buf);
            // filter off slots in memory that have no value stored
            if buf[..EVENT_SIZE] != UNUSED_ENTRY {
                let event_id = EventId::from_bytes(&buf[..EVENT_SIZE]);
                self.inner.set_event_item(
                    event_id,
                    HeaplessLearnedEvent { index: index as u8, vars: Vec::from_slice(&buf[EVENT_SIZE..]).unwrap()}
                );
            }
        }
    }

    /// Checks if the module is in it's first setup
    ///
    /// This is done by comparing values read in the [`PERSISTENT_BLOCK_SIZE`] from the [`OFFSET`].
    /// At the moment the method expects all values in the block to have value of `0xFF`
    fn detect_virgin_storage_state(&mut self) -> bool {
        let mut storage = self.driver.borrow_mut();

        // SAFETY: get block of memory for readout, we don't care about initializing it
        #[allow(unsafe_code, clippy::uninit_assumed_init)]
        let mut buf = unsafe {[const { MaybeUninit::<u8>::uninit().assume_init() }; PERSISTENT_BLOCK_SIZE as usize]};

        // TODO: maybe instead just compare mode and node num ranges?
        let _ = storage.read(0, &mut buf);

        buf.iter().all(|v| *v == UNINITIALISED_VALUE)
    }

    /// Reloads node variables from persistent memory
    fn reload_nv(&mut self) {
        let mut storage = self.driver.borrow_mut();

        // SAFETY: get block of memory for readout, we don't care about initializing it
        #[allow(unsafe_code, clippy::uninit_assumed_init)]
        let mut buf = unsafe {[const { MaybeUninit::<u8>::uninit().assume_init() }; 1]};

        for (index, addr) in (Self::nv_addr_start()..=Self::nv_addr_end()).enumerate() {
            let _ = storage.read(addr as u32, &mut buf);
            self.inner.set_nv((index + 1) as u8, buf[0]).unwrap();
        }
    }

    #[inline]
    fn mark_as_dirty(&mut self) -> &mut NodeConfigStorage<MAX_EVENTS, EVENT_VAR_COUNT, NODE_VAR_COUNT> {
        self.dirty = true;
        &mut self.inner
    }

    fn flush_to_storage(&mut self) {
        let mut storage = self.driver.borrow_mut();

        // the memory block should be as big as the biggest chunk we are going to read
        // SAFETY: get block of memory for readout, we don't care about initializing it
        #[allow(unsafe_code, clippy::uninit_assumed_init)]
        let mut buf = unsafe {[const { MaybeUninit::<u8>::uninit().assume_init() }; { cmax(1, cmax(CANID_SIZE, NODENUM_SIZE)) }]};

        // readout the mode and save if the current mode is different from the stored one
        let _ = storage.read(Self::mode_addr() as u32, &mut buf[..1]);
        {
            let mode = self.inner.mode() as u8;
            if mode != buf[0] {
                buf[0] = mode;
                let _ = storage.write(Self::mode_addr() as u32, &buf[..1]);
            }
        }

        // if the current mode is NORMAL we can store the current node number if it's different
        // ignore otherwise as it's considered as trash values and it won't be loaded
        if self.mode() == VlcbModeParams::NORMAL {
            // read out the stored node number
            let _ = storage.read(Self::node_num_addr_start() as u32, &mut buf[..NODENUM_SIZE]);
            let node_num = self.inner.node_number().as_bytes();
            if buf[..NODENUM_SIZE] != *node_num {
                buf[..NODENUM_SIZE].copy_from_slice(node_num);
                let _ = storage.write(Self::node_num_addr_start() as u32, &buf[..NODENUM_SIZE]);
            }
        }

        // save the flags if they differ from persisted values
        let _ = storage.read(Self::flags_addr() as u32, &mut buf[..1]);
        {
            let bits = self.inner.flags().bits();
            if bits != buf[0] {
                buf[0] = bits;
                let _ = storage.write(Self::flags_addr() as u32, &buf[..1]);
            }
        }

        // store the can_id
        let _ = storage.read(Self::can_id_addr() as u32, &mut buf[..CANID_SIZE]);
        {
            let can_id = self.inner.can_id().as_bytes();
            if buf[..CANID_SIZE] != *can_id {
                buf[..CANID_SIZE].copy_from_slice(can_id);
                let _ = storage.write(Self::can_id_addr() as u32, &buf);
            }
        }

        // save the reset flag
        let _ = storage.read(Self::reset_flag_addr() as u32, &mut buf[..1]);
        {
            let flag = match self.inner.was_reset() {
                true => FLAGGED_AS_RESET,
                false => RESET_FLAG_CLEARED,
            };
            if buf[0] != flag {
                buf[0] = flag;
                let _ = storage.write(Self::reset_flag_addr() as u32, &buf[..1]);
            }
        }
    }
}

impl<
        D: StorageDriver,
        const OFFSET: usize,
        const MAX_EVENTS: usize,
        const EVENT_VAR_COUNT: usize,
        const BYTES_PER_EVENT: usize,
        const NODE_VAR_COUNT: usize,
    > PersistentStorage for PersistentNodeConfigStorage<D, OFFSET, MAX_EVENTS, EVENT_VAR_COUNT, BYTES_PER_EVENT, NODE_VAR_COUNT>
{
    #[allow(clippy::must_use_unit)]
    #[must_use]
    fn load(&mut self) {
        {
            if  self.detect_virgin_storage_state() {
                self.clear_reset_flag();
                self.force_flush();
            }

            let mut storage = self.driver.borrow_mut();

            // the memory block should be as big as the biggest chunk we are going to read
            // SAFETY: get block of memory for readout, we don't care about initializing it
            #[allow(unsafe_code, clippy::uninit_assumed_init)]
            let mut buf = unsafe {[const { MaybeUninit::<u8>::uninit().assume_init() }; { cmax(1, cmax(CANID_SIZE, NODENUM_SIZE)) }]};

            // readout the mode and initialize the mode based on it's current status
            let _ = storage.read(Self::mode_addr() as u32, &mut buf[..1]);
            match VlcbModeParams::from(buf[0]) {
                VlcbModeParams::NORMAL => {
                    // read out the stored node number
                    let _ = storage.read(Self::node_num_addr_start() as u32, &mut buf[..NODENUM_SIZE]);
                    self.inner.set_mode_normal(VlcbNodeNumber::from_bytes(&buf[..NODENUM_SIZE]))
                },
                _ => self.inner.set_mode_uninitialized(),// other modes are unsupported here
            }

            // read out the flags or set the value to default (empty)
            let _ = storage.read(Self::flags_addr() as u32, &mut buf[..1]);
            self.inner.set_flags(NodeFlags::from_bits(buf[0]).unwrap_or(NodeFlags::empty()));

            // read out the stored can_id
            let _ = storage.read(Self::can_id_addr() as u32, &mut buf[..CANID_SIZE]);
            self.inner.set_can_id(VlcbCanId::from_bytes(&buf[..CANID_SIZE]));

            // read out the reset flag position and check if it has been set
            let _ = storage.read(Self::reset_flag_addr() as u32, &mut buf[..1]);
            if buf[0] == FLAGGED_AS_RESET {
                self.inner.raise_reset_flag();
            }
        }

        self.reload_event_hash_table();
        self.reload_nv();
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn flush(&mut self) {
        if ! self.dirty {
            return
        }

        self.flush_to_storage();

        self.dirty = false
    }

    fn force_flush(&mut self) {
        self.flush_to_storage();
    }
}

impl<
        D: StorageDriver,
        const OFFSET: usize,
        const MAX_EVENTS: usize,
        const EVENT_VAR_COUNT: usize,
        const BYTES_PER_EVENT: usize,
        const NODE_VAR_COUNT: usize,
    > NodeConfig for PersistentNodeConfigStorage<D, OFFSET, MAX_EVENTS, EVENT_VAR_COUNT, BYTES_PER_EVENT, NODE_VAR_COUNT>
{
    type Event = HeaplessLearnedEvent<EVENT_VAR_COUNT>;
    const MAX_EVENTS: u8 = MAX_EVENTS as u8;
    const EVENT_VAR_COUNT: u8 = EVENT_VAR_COUNT as u8;
    const NODE_VAR_COUNT: u8 = NODE_VAR_COUNT as u8;

    delegate! {
        to self.inner {
            fn stored_event_count(&self) -> u8;
            fn has_event_with_index(&self, index: u8) -> bool;
            fn get_event(&self, evt: &EventId) -> Option<&Self::Event>;
            fn has_event(&self, evt: &EventId) -> bool;
            fn get_nv(&self, index: u8) -> Result<u8, Error>;
            fn can_id(&self) -> &VlcbCanId;
            fn mode(&self) -> VlcbModeParams;
            fn node_number(&self) -> &VlcbNodeNumber;
            fn was_reset(&self) -> bool;
            fn is_heartbeat_on(&self) -> bool;
            fn is_event_ack_on(&self) -> bool;
            fn flags(&self) -> NodeFlags;
        }
        // Mutations should mark this implementation as dirty so it can be flushed to storage
        to self.mark_as_dirty() {
            fn save_event(&mut self, evt: &EventId, evs: &[u8]) -> Result<(), Error>;
            fn restore_event(&mut self, evt: EventId, data: Self::Event) -> Result<(), Error>;
            fn restore_event_unchecked(&mut self, evt: EventId, data: Self::Event) -> Result<(), Error>;
            fn delete_event(&mut self, evt: &EventId);
            fn set_nv(&mut self, index: u8, value: u8) -> Result<(), Error>;
            fn set_can_id(&mut self, can_id: VlcbCanId);
            fn set_mode_normal(&mut self, node_num: VlcbNodeNumber);
            fn set_mode_uninitialized(&mut self);
            fn set_node_number(&mut self, node_num: VlcbNodeNumber);
            fn raise_reset_flag(&mut self);
            fn clear_reset_flag(&mut self);
            fn set_heartbeat(&mut self, state: bool);
            fn set_event_ack(&mut self, state: bool);
            fn set_flags(&mut self, flags: NodeFlags);
        }
    }
}

impl<
        D: StorageDriver,
        const OFFSET: usize,
        const MAX_EVENTS: usize,
        const EVENT_VAR_COUNT: usize,
        const BYTES_PER_EVENT: usize,
        const NODE_VAR_COUNT: usize,
    > Storage for PersistentNodeConfigStorage<D, OFFSET, MAX_EVENTS, EVENT_VAR_COUNT, BYTES_PER_EVENT, NODE_VAR_COUNT>
{
    fn wipe(&mut self) {
        self.inner.wipe();
        self.dirty = true;
        self.flush();
    }
}