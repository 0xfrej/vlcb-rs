#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![deny(unsafe_code)]

use cfg_if::cfg_if;
use service_set::ServiceSet;
use vlcb_persistence::node_config::NodeConfig;
use vlcb_persistence::PersistentStorage;
use embedded_time::{Clock, Instant};

use vlcb_defs::{
    CbusArmProcessors, CbusBusTypes, CbusManufacturer, CbusMergModuleTypes, CbusMicrochipProcessors, CbusParams, CbusProcessorManufacturers
};
use vlcb_network::iface::{Interface, SocketSet};
use vlcb_network::phy::{Device};

use vlcb_ui::VlcbUi;

const MODULE_PARAMS_COUNT: usize = 20;

pub mod service_set;

pub type CpuId = [char; 4];
pub type CpuIdResolver = fn() -> CpuId;

// pub enum ModuleType {
//     Merg(CbusMergModuleTypes),
//     Sprog(CbusSprogModuleTypes),
//     RocRail(CbusRocRailModuleTypes),
//     Spectrum(CbusSpectrumModuleTypes),
//     SysPixie(CbusSysPixieModuleTypes),
//     Generic(u8),
// }

// impl Into<u8> for ModuleType {
//     fn into(self) -> u8 {
//         match self {
//             Self::Merg(v) => v.into(),
//             Self::Sprog(v) => v.into(),
//             Self::RocRail(v) => v.into(),
//             Self::Spectrum(v) => v.into(),
//             Self::SysPixie(v) => v.into(),
//             Self::Generic(v) => v,
//         }
//     }
// }

pub enum Processor {
    Arm(CbusArmProcessors),
    Microchip(CbusMicrochipProcessors),
    Atmel,
}

impl Processor {
    fn emit(self, params: &mut ModuleParams) {
        match self {
            Self::Arm(id) => {
                params.set_param(CbusParams::CPUID, id as u8);
                params.set_param(CbusParams::CPUMAN, CbusProcessorManufacturers::ARM as u8)
            }
            Self::Microchip(id) => {
                params.set_param(CbusParams::CPUID, id as u8);
                params.set_param(
                    CbusParams::CPUMAN,
                    CbusProcessorManufacturers::MICROCHIP as u8,
                )
            }
            Self::Atmel => {
                params.set_param(CbusParams::CPUID, 50);
                params.set_param(CbusParams::CPUMAN, CbusProcessorManufacturers::ATMEL as u8)
            }
        };
    }
}

impl From<Processor> for CbusProcessorManufacturers {
    fn from(value: Processor) -> Self {
        match value {
            Processor::Arm(_) => CbusProcessorManufacturers::ARM,
            Processor::Microchip(_) => CbusProcessorManufacturers::MICROCHIP,
            Processor::Atmel => CbusProcessorManufacturers::ATMEL,
        }
    }
}

impl TryFrom<Processor> for CbusArmProcessors {
    type Error = ();

    fn try_from(value: Processor) -> Result<Self, Self::Error> {
        match value {
            Processor::Arm(v) => Ok(v),
            _ => Err(()),
        }
    }
}

impl TryFrom<Processor> for CbusMicrochipProcessors {
    type Error = ();

    fn try_from(value: Processor) -> Result<Self, Self::Error> {
        match value {
            Processor::Microchip(v) => Ok(v),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct ModuleVersion {
    major: u8,
    minor: char,
    beta: u8,
}

impl ModuleVersion {
    pub fn new(major: u8, minor: char, beta: u8) -> Self {
        debug_assert!(
            minor.is_ascii_alphabetic(),
            "The minor version must be a ASCII alphabetic character"
        );
        Self { major, minor, beta }
    }

    fn emit(self, params: &mut ModuleParams) {
        params.set_param(CbusParams::MAJVER, self.major);
        params.set_param(CbusParams::MINVER, self.minor as u8);
        params.set_param(CbusParams::BETA, self.beta);
    }
}

#[derive(Default, Debug)]
struct ModuleParams([u8; MODULE_PARAMS_COUNT]);
impl ModuleParams {
    pub(crate) fn new(cpu: Processor, cpu_id_resolver: Option<CpuIdResolver>) -> Self {
        let mut params = Self([0; MODULE_PARAMS_COUNT]);

        cpu.emit(&mut params);

        if let Some(r) = cpu_id_resolver {
            let name: heapless::Vec<u8, 4> = r()
                .as_slice()
                .iter()
                .map(|v| {
                    debug_assert!(v.is_ascii(), "all characters need to be ASCII");
                    *v as u8
                })
                .collect();
            params.0[(CbusParams::CPUMID as usize)..4].copy_from_slice(name.as_slice());
        } else {
            params.0[(CbusParams::CPUMID as usize)..4].copy_from_slice([b'?'; 4].as_slice());
        }

        params
    }

    pub(crate) fn get_param(&self, param: CbusParams) -> u8 {
        self.0[(param as usize) - 1]
    }

    pub(crate) fn set_param(&mut self, param: CbusParams, value: u8) {
        self.0[(param as usize) - 1] = value
    }
}

pub struct Module<UI: VlcbUi<C>, C: Clock, S: NodeConfig> {
    name: &'static str,
    params: ModuleParams,
    inner: ModuleInner<UI, C, S>,
}

struct ModuleInner<UI: VlcbUi<C>, C: Clock, S: NodeConfig> {
    now: Instant<C>,
    config: S,
    ui: UI,
    interface: Interface<C>,
}

impl<UI: VlcbUi<C>, C: Clock, S: NodeConfig + PersistentStorage>
    Module<UI, C, S>
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &'static str,
        version: ModuleVersion,
        manufacturer: CbusManufacturer,
        flags: u8,
        ui: UI,
        config: S,
        cpu: Processor,
        cpu_id_resolver: Option<CpuIdResolver>,
        interface: Interface<C>,
        services: &ServiceSet
    ) -> Self {
        let mut params = ModuleParams::new(cpu, cpu_id_resolver);

        params.set_param(CbusParams::MTYP, CbusMergModuleTypes::VLCB.into());
        params.set_param(CbusParams::FLAGS, flags);

        version.emit(&mut params);

        params.set_param(CbusParams::MANU, manufacturer.into());
        params.set_param(
            CbusParams::BUSTYPE,
            CbusBusTypes::from(interface.device_caps().medium).into(),
        );

        params.set_param(CbusParams::EVTNUM, S::MAX_EVENTS);
        params.set_param(CbusParams::EVNUM, S::EVENT_VAR_COUNT);
        params.set_param(CbusParams::NVNUM, S::NODE_VAR_COUNT);

        Self {
            name,
            params,
            inner: ModuleInner {
                now: Instant::new(C::T::from(0)),
                config,
                ui,
                interface,
            },
        }
    }

    /// Initialize the module instance
    ///
    /// Loads config data from memory, and restores the saved state from previous runs if supported.
    /// Restores the interface addresses from memory.
    pub fn init(mut self) -> Self {
        todo!();
        // self.inner.config.load();

        // let addr: VlcbNodeNumber;
        // #[cfg(feature = "medium-can")]
        // let can_id: VlcbCanId;
        // if self.inner.config.flim() {
        //     addr = self.inner.config.node_number();

        //     #[cfg(feature = "medium-can")]
        //     {
        //         can_id = self.inner.config.can_id();
        //     }
        // } else {
        //     addr = VlcbNodeNumber::default();
        //     can_id = VlcbCanId::default();
        // }

        // iface.set_addr(addr);
        // #[cfg(feature = "medium-can")]
        // {
        //     if Medium::CAN == iface.device_caps().medium {
        //         iface.set_hw_addr(HardwareAddress::CAN(can_id));
        //     }
        // }

        // Module {
        //     name: self.name,
        //     params: self.params,
        //     inner: self.inner,
        // }
    }
}

impl<UI: VlcbUi<C>, C: Clock, S: NodeConfig + PersistentStorage>
    Module<UI, C, S>
{
    /// Shutdown the module
    ///
    /// Ensures finalization of ongoing activities, flushing unsaved states to persistent memory if
    /// required, etc.
    ///
    /// Does not ensure packets are
    /// TODO: Should we ensure flushing leftover packets?
    pub fn shutdown(self) -> Self {
        todo!();
        Module {
            name: self.name,
            params: self.params,
            inner: self.inner,
        }
    }

    pub fn reset_module(&mut self) {
        // /// standard implementation of resetModule()
        //
        // bool bDone;
        // unsigned long waittime;
        //
        // // start timeout timer
        // waittime = millis();
        // bDone = false;
        //
        // // DEBUG_SERIAL << F("> waiting for a further 5 sec button push, as a safety measure") << endl;
        //
        // pbSwitch.reset();
        // ledGrn.blink();
        // ledYlw.blink();
        //
        // // wait for a further (5 sec) button press -- as a 'safety' mechanism
        // while (!bDone) {
        //
        //     // 30 sec timeout
        //     if ((millis() - waittime) > 30000) {
        //         // DEBUG_SERIAL << F("> timeout expired, reset not performed") << endl;
        //         return;
        //     }
        //
        //     pbSwitch.run();
        //     ledGrn.run();
        //     ledYlw.run();
        //
        //     // wait until switch held for a further 5 secs
        //     if (pbSwitch.isPressed() && pbSwitch.getCurrentStateDuration() > 5000) {
        //         bDone = true;
        //     }
        // }
        //
        // // do the reset
        // // DEBUG_SERIAL << F("> performing module reset ...") <<  endl;
        //
        // ledGrn.off();
        // ledYlw.off();
        // ledGrn.run();
        // ledYlw.run();
        //
        // resetModule(); -> this is basically continuation to the next lines reseting the data

        // self.config.wipe();
        // self.config.set_flim(false);
        // self.config.set_can_id(CbusCanId::default());
        // self.config.set_node_number(CbusNodeNumber::default());
        // self.config.flag_for_reset();
    }

    pub fn poll<'a, D: Device>(
        &mut self,
        now: Instant<C>,
        interface: &'a mut Interface<C>,
        device: &'a mut D,
        sockets: &'a mut SocketSet<'a>,
    ) {
        self.inner.now = now;

        // TODO: module stuff like flim, can enumeration etc should be done using a socket
        // the socket impl should only forward packets we care about and then processing here should
        // use the socket to reply back either by responding to can enumeration, flim stuff etc
        // the socket can be essentially just filtered raw cbus socket

        // self.process_mode_state(interface);

        // TODO: instead of forcing the library users to adhere to this logic it should be rewriten to "on request"
        // so that users can manipulate the button behaviors and things and maybe implement a default loop elsewhere
        // also makes this more testable i guess
        cfg_if! {
            if #[cfg(feature = "user-interface")] {
                self.inner.ui.poll(now)

                /*

          // use LEDs to indicate that the user can release the switch
            if (_sw.isPressed() && _sw.getCurrentStateDuration() > SW_TR_HOLD) {
                indicateMode(MODE_CHANGING);
            }

          //
          /// handle switch state changes
          //

          if (_sw.stateChanged()) {

            // has switch been released ?
            if (!_sw.isPressed()) {

              // how long was it pressed for ?
              unsigned long press_time = _sw.getLastStateDuration();

              // long hold > 6 secs
              if (press_time > SW_TR_HOLD) {
                // initiate mode change
                if (!module_config->FLiM) {
                  initFLiM();
                } else {
                  revertSLiM();cbus::Packet
                }
              }

              // short 1-2 secs
              if (press_time >= 1000 && press_time < 2000) {
                renegotiate();
              }

              // very short < 0.5 sec
              if (press_time < 500 && module_config->FLiM) {
                CANenumeration();
              }

            } else {
              // do any switch release processing here
            }
          }
        }
        */
            }
        }

        // sockets.

        // let ctx: PollContext<'a, D, C> = PollContext::new(now, device, sockets);
        todo!();
        // interface.poll(ctx);
    }
}