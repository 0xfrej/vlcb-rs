extern crate vlcb_module;

use std::cell::RefCell;

use rclite::Rc;
use vlcb_macros::str_to_array;
use vlcb_module::{CpuId, CpuIdResolver, Module, ModuleVersion};
use vlcb_module_macros::module_version;
use vlcb_network::iface::Interface;
use vlcb_persistence::{node_config::PersistentNodeConfigStorage};
use embedded_storage_inmemory::MemFlash;

fn processor_id_resolver() -> CpuId {
    str_to_array!("328P")
}

fn main() -> ! {
    // Real module should use EEPROM or flash or similar for persistence
    let storage_driver = Rc::new(RefCell::new(MemFlash::<128,1,1>::new(0xff)));

    // Init config to start of the memory pointed at by `storage_driver`
    // The inmemory storage uses array buffer, but for usually this should be an address at which
    // the config block storage should start.

    // currently there is a limitation in rust language with solutions in unstable rust
    // we can't use const expressions in generics, therefore the user of [`PersistentNodeConfigStorage`] must
    // provide the value of `BYTES_PER_EVENT`, which can be computed using a helper function [`bytes_per_event`]
    // which takes in number of event vars as an argument. Or manually inputting
    const EVENT_VARS: usize = 4;
    let mut config = PersistentNodeConfigStorage::<_, 0, 32, EVENT_VARS, bytes_per_event(EVENT_VARS), 32>::new(storage_driver.clone());
    
    let interface = Interface::new(device, addr, hw_addr);

    let mut module = Module::new(
        "My Little Test Module",
        // module_version!("1.a.33"),
        ModuleVersion::new(1, 'a', 33),
        vlcb_defs::CbusManufacturer::DEV,
        0,
        ui,
        config,
        vlcb_module::Processor::Atmel,
        Some(processor_id_resolver),
        interface,
        services
    );

    loop {

    }
}