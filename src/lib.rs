use std::{thread::sleep, collections::HashMap};

use asr::{Process, watcher::Pair, signature::Signature, Address};
use spinning_top::{Spinlock, const_spinlock};
use once_cell::sync::Lazy;

const MAIN_MODULE: &str = "HatinTimeGame.exe";

fn update_pair<T: std::fmt::Display + Copy>(variable_name: &str, new_value: T, pair: &mut Pair<T>) {
    asr::timer::set_variable(variable_name, &format!("{new_value}"));
    pair.old = pair.current;
    pair.current = new_value;
}

#[derive(PartialEq, Debug)]
enum PatchType {
    Unknown,
    DLC,
    CDLC,
    Modding,
    Release,
}

#[derive(Default)]
struct MemoryAddresses {
    timr: Option<Address>, // gives access to main timer values
    save_data_base: Option<Address>, // base of the pointer path that leads to save data
    save_data_offsets: HashMap<String, [u64; 2]>,
}

#[derive(Default)]
struct MemoryValues {
    // main timer values
    timer_state: Pair<i32>, // 0 for inactive, 1 for orange running, 2 for green stopped
    unpause_time: Pair<f64>, // loading screen duration
    game_timer_is_paused: Pair<i32>,
    act_timer_is_paused: Pair<i32>,
    act_timer_is_visible: Pair<i32>,
    unpause_timer_is_dirty: Pair<i32>, // idk
    just_got_time_piece: Pair<i32>, // green act timer -> 1
    game_time: Pair<f64>, // load screen times
    act_time: Pair<f64>,
    real_game_time: Pair<f64>, // running times
    real_act_time: Pair<f64>,
    tp_count: Pair<i32>,
    // save data values
    yarn:  Pair<i32>,
    chapter: Pair<i32>,
    act: Pair<i32>,
    checkpoint: Pair<i32>,
}

struct State {
    started_up: bool,
    main_process: Option<Process>,
    values: Lazy<MemoryValues>,
    addresses: Lazy<MemoryAddresses>,
    patch_type: PatchType,
}

impl State {

    fn startup(&mut self) {
        asr::set_tick_rate(10.0);
        self.started_up = true;
    }

    fn init(&mut self) -> Result<(), &str> {
        // idk just some random ass number, TODO: do it like og ls when possible, it iterates through the memory pages
        let size = 0x3200000;

        let process = self.main_process.as_ref().unwrap();
        let hat_main_add = process.get_module_address(MAIN_MODULE).unwrap();

        // scan for the IGT values
        const TIMR_AOB: Signature<76> = Signature::new(
            concat!(
            "54 49 4D 52", // TIMR
            "?? ?? ?? ??", // timerState
            "?? ?? ?? ?? ?? ?? ?? ??", // unpauseTime
            "?? ?? ?? ??", // gameTimerIsPaused
            "?? ?? ?? ??", // actTimerIsPaused
            "?? ?? ?? ??", // actTimerIsVisible
            "?? ?? ?? ??", // unpauseTimeIsDirty
            "?? ?? ?? ??", // justGotTimePiece
            "?? ?? ?? ?? ?? ?? ?? ??", // gameTime
            "?? ?? ?? ?? ?? ?? ?? ??", // actTime
            "?? ?? ?? ?? ?? ?? ?? ??", // realGameTime
            "?? ?? ?? ?? ?? ?? ?? ??", // realActTime
            "?? ?? ?? ??", // timePieceCount
            "45 4E 44 20" // END
            )
        );

        let scan_result_address = TIMR_AOB.scan_process_range(process, hat_main_add, size);
        match scan_result_address {
            Some(scan_result) => self.addresses.timr = Some(Address(scan_result.0 - hat_main_add.0)),
            None => return Err("Could not find TIMR address"),
        }

        
        // scan for the save file data base address for the pointer path
        // it will scan until one signature works (if you see this please tell me how i could make this shorter, my brain was fried while making this loopless)
        let mut scan_result: Option<Address>;
        const SAVE_DATA_BASE_AOB_DLC2: Signature<16> = Signature::new("48 8B 05 ?? ?? ?? ?? 48 8B 74 24 ?? 48 83 C4 50");
        scan_result = SAVE_DATA_BASE_AOB_DLC2.scan_process_range(process, hat_main_add, size);

        if scan_result.is_some() && self.patch_type == PatchType::Unknown {
            self.patch_type = PatchType::DLC;
        } else if scan_result.is_none() {
            const SAVE_DATA_AOB_VACU: Signature<21> = Signature::new("48 8B 1D ?? ?? ?? ?? 48 85 DB 74 ?? 48 8B 5B ?? 48 85 DB 74 ??");
            scan_result = SAVE_DATA_AOB_VACU.scan_process_range(process, hat_main_add, size);
        }

        if scan_result.is_some() && self.patch_type == PatchType::Unknown {
            self.patch_type = PatchType::CDLC;
        } else if scan_result.is_none() {
            const SAVE_DATA_AOB_MODDING: Signature<20> = Signature::new("48 8B 05 ?? ?? ?? ?? 48 8B D9 48 85 C0 75 ?? 48 89 7C 24 ??");
            scan_result = SAVE_DATA_AOB_MODDING.scan_process_range(process, hat_main_add, size);
        }

        if scan_result.is_some() && self.patch_type == PatchType::Unknown {
            self.patch_type = PatchType::Modding;
        } else if scan_result.is_none() {
            const SAVE_DATA_AOB_RELEASE: Signature<16> = Signature::new("48 8B 05 ?? ?? ?? ?? 48 8B 7C 24 ?? 48 83 C4 40");
            scan_result = SAVE_DATA_AOB_RELEASE.scan_process_range(process, hat_main_add, size);
        }

        if scan_result.is_some() && self.patch_type == PatchType::Unknown {
            self.patch_type = PatchType::Release;
        } else if scan_result.is_none() {
            return Err("Could not find Save Data base address with sigscan");
        }

        if scan_result.is_some() {
            // read data in sigscan, it's an offset for the address found itself which will lead to the save data base pointer
            let save_data_offset = process.read::<u32>(Address(scan_result.unwrap().0 + 0x3)).unwrap() as u64;
            // final address is just an addition of the sigscan and the offset, with a slight nudge to land in the correct place with the save data
            let final_adress = scan_result.unwrap().0 + save_data_offset + 0x7;
            self.addresses.save_data_base = Some(Address(final_adress - hat_main_add.0));
        }

        // offsets for the save data pointer path
        match self.patch_type {
            PatchType::Unknown => {
                // just in case... the save files haven't changed in a while anyway so this is the same as CDLC
                self.addresses.save_data_offsets.insert("yarn".to_owned(), [0x68, 0xF0]);
                self.addresses.save_data_offsets.insert("chapter".to_owned(), [0x68, 0x108]);
                self.addresses.save_data_offsets.insert("act".to_owned(), [0x68, 0x10C]);
                self.addresses.save_data_offsets.insert("checkpoint".to_owned(), [0x68, 0x110]);
            },
            PatchType::DLC => {
                self.addresses.save_data_offsets.insert("yarn".to_owned(), [0x68, 0xF0]);
                self.addresses.save_data_offsets.insert("chapter".to_owned(), [0x68, 0x108]);
                self.addresses.save_data_offsets.insert("act".to_owned(), [0x68, 0x10C]);
                self.addresses.save_data_offsets.insert("checkpoint".to_owned(), [0x68, 0x110]);
            },
            PatchType::CDLC => {
                self.addresses.save_data_offsets.insert("yarn".to_owned(), [0x68, 0xF0]);
                self.addresses.save_data_offsets.insert("chapter".to_owned(), [0x68, 0x108]);
                self.addresses.save_data_offsets.insert("act".to_owned(), [0x68, 0x10C]);
                self.addresses.save_data_offsets.insert("checkpoint".to_owned(), [0x68, 0x110]);
            },
            PatchType::Modding => {
                self.addresses.save_data_offsets.insert("yarn".to_owned(), [0x64, 0xE0]);
                self.addresses.save_data_offsets.insert("chapter".to_owned(), [0x64, 0xF8]);
                self.addresses.save_data_offsets.insert("act".to_owned(), [0x64, 0xFC]);
                self.addresses.save_data_offsets.insert("checkpoint".to_owned(), [0x64, 0x100]);
            },
            PatchType::Release => {
                self.addresses.save_data_offsets.insert("yarn".to_owned(), [0x64, 0xE0]);
                self.addresses.save_data_offsets.insert("chapter".to_owned(), [0x64, 0xF4]);
                self.addresses.save_data_offsets.insert("act".to_owned(), [0x64, 0xF8]);
                self.addresses.save_data_offsets.insert("checkpoint".to_owned(), [0x64, 0xFC]);
            },
        }

        asr::timer::set_variable("Patch Type", &format!("{:?}", self.patch_type));
        

        asr::set_tick_rate(120.0);
        
        Ok(())

    }

    fn refresh_mem_values(&mut self) -> Result<(), &str> {

        let main_module_addr = match &self.main_process {
            Some(info) => match info.get_module_address(MAIN_MODULE) {
                Ok(address) => address,
                Err(_) => return Err("Could not get module address when refreshing memory values.")
            },
            None => return Err("Process info is not initialized.")
        };

        let process = self.main_process.as_ref().unwrap();

        // memory reads

        // TIMR
        // timer state
        if let Ok(value) = process.read_pointer_path64::<i32>(main_module_addr.0, &[self.addresses.timr.unwrap().0 + 0x4]) {
            update_pair("Timer State", value, &mut self.values.timer_state);
        };

        // unpause time
        if let Ok(value) = process.read_pointer_path64::<f64>(main_module_addr.0, &[self.addresses.timr.unwrap().0 + 0x8]) {
            update_pair("Unpause Time", value, &mut self.values.unpause_time);
        };

        // game timer is paused
        if let Ok(value) = process.read_pointer_path64::<i32>(main_module_addr.0, &[self.addresses.timr.unwrap().0 + 0x10]) {
            update_pair("Game Timer Is Paused", value, &mut self.values.game_timer_is_paused);
        };


        // act timer is paused
        if let Ok(value) = process.read_pointer_path64::<i32>(main_module_addr.0, &[self.addresses.timr.unwrap().0 + 0x14]) {
            update_pair("Act Timer Is Paused", value, &mut self.values.act_timer_is_paused);
        };

        // act timer is visible
        if let Ok(value) = process.read_pointer_path64::<i32>(main_module_addr.0, &[self.addresses.timr.unwrap().0 + 0x18]) {
            update_pair("Act timer is visible", value, &mut self.values.act_timer_is_visible);
        };

        // unpause time is dirty (idk what this is)
        if let Ok(value) = process.read_pointer_path64::<i32>(main_module_addr.0, &[self.addresses.timr.unwrap().0 + 0x1C]) {
            update_pair("Unpause time is dirty", value, &mut self.values.unpause_timer_is_dirty);
        };

        // just got time piece (green act time)
        if let Ok(value) = process.read_pointer_path64::<i32>(main_module_addr.0, &[self.addresses.timr.unwrap().0 + 0x20]) {
            update_pair("Just Got Time Piece", value, &mut self.values.just_got_time_piece);
        };

        // game time (loading screens)
        if let Ok(value) = process.read_pointer_path64::<f64>(main_module_addr.0, &[self.addresses.timr.unwrap().0 + 0x24]) {
            update_pair("Game Time (Loads)", value, &mut self.values.game_time);
        };

        // act time (loading screens)
        if let Ok(value) = process.read_pointer_path64::<f64>(main_module_addr.0, &[self.addresses.timr.unwrap().0 + 0x2C]) {
            update_pair("Act Time (Loads)", value, &mut self.values.act_time);
        };

        // game time (real/running)
        if let Ok(value) = process.read_pointer_path64::<f64>(main_module_addr.0, &[self.addresses.timr.unwrap().0 + 0x34]) {
            update_pair("Game Time", value, &mut self.values.real_game_time);
        };

        // act time (real/running)
        if let Ok(value) = process.read_pointer_path64::<f64>(main_module_addr.0, &[self.addresses.timr.unwrap().0 + 0x3C]) {
            update_pair("Act Time", value, &mut self.values.real_act_time);
        };

        // time piece count
        if let Ok(value) = process.read_pointer_path64::<i32>(main_module_addr.0, &[self.addresses.timr.unwrap().0 + 0x44]) {
            update_pair("Time Pieces", value, &mut self.values.tp_count);
        };

        // save data
        // yarn
        let offsets = self.addresses.save_data_offsets["yarn"];
        if let Ok(value) = process.read_pointer_path64::<i32>(main_module_addr.0, &[self.addresses.save_data_base.unwrap().0, offsets[0], offsets[1]]) {
            update_pair("Yarn", value, &mut self.values.yarn);
        };

        // chapter
        let offsets = self.addresses.save_data_offsets["chapter"];
        if let Ok(value) = process.read_pointer_path64::<i32>(main_module_addr.0, &[self.addresses.save_data_base.unwrap().0, offsets[0], offsets[1]]) {
            update_pair("Chapter", value, &mut self.values.chapter);
        };

        // act
        let offsets = self.addresses.save_data_offsets["act"];
        if let Ok(value) = process.read_pointer_path64::<i32>(main_module_addr.0, &[self.addresses.save_data_base.unwrap().0, offsets[0], offsets[1]]) {
            update_pair("Act", value, &mut self.values.act);
        };

        // checkpoint
        let offsets = self.addresses.save_data_offsets["checkpoint"];
        if let Ok(value) = process.read_pointer_path64::<i32>(main_module_addr.0, &[self.addresses.save_data_base.unwrap().0, offsets[0], offsets[1]]) {
            update_pair("Checkpoint", value, &mut self.values.checkpoint);
        };

        Ok(())
    }

    fn update(&mut self) {

        if !self.started_up {
            self.startup();
        }

        if self.main_process.is_none() {
            self.main_process = Process::attach(MAIN_MODULE);
            if self.main_process.is_some() {
                // run init, remove process if something went wrong in it
                if let Err(message) = self.init() {
                    asr::print_message(&format!("ERROR: init() didn't finish properly, message: {}", message));
                    self.main_process = None;
                    sleep(std::time::Duration::from_secs(2));
                    return;
                }
            }
            // early return to never work with a None process
            return;
        }

        // if game is closed detatch and look for the game again
        if !self.main_process.as_ref().unwrap().is_open() {
            asr::set_tick_rate(10.0);
            self.main_process = None;
            self.patch_type = PatchType::Unknown;
            return;
        }

        if self.refresh_mem_values().is_err() {
            return;
        }

        asr::timer::set_game_time(asr::time::Duration::seconds_f64(self.values.real_game_time.current));

    }

}

static LS_CONTROLLER: Spinlock<State> = const_spinlock(State {
    started_up: false,
    main_process: None,
    values: Lazy::new(Default::default),
    addresses: Lazy::new(Default::default),
    patch_type: PatchType::Unknown,
});


#[no_mangle]
pub extern "C" fn update() {
    LS_CONTROLLER.lock().update();
}
