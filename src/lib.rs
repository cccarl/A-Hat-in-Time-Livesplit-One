use asr::{Process, watcher::Pair, signature::Signature, Address};
use spinning_top::{Spinlock, const_spinlock};
use once_cell::sync::Lazy;

const MAIN_MODULE: &str = "HatinTimeGame.exe";

fn update_pair<T: std::fmt::Display + Copy>(variable_name: &str, new_value: T, pair: &mut Pair<T>) {
    asr::timer::set_variable(variable_name, &format!("{new_value}"));
    pair.old = pair.current;
    pair.current = new_value;
}

struct MemoryAddresses {
    timr: Option<Address>, // gives access to main timer values
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
}

struct State {
    started_up: bool,
    main_process: Option<Process>,
    values: Lazy<MemoryValues>,
    addresses: MemoryAddresses,
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

        /*
        const SAVE_DATA_AOB_VACU: Signature<21> = Signature::new("48 8B 1D ?? ?? ?? ?? 48 85 DB 74 ?? 48 8B 5B ?? 48 85 DB 74 ??");
        SAVE_DATA_AOB_VACU.scan_process_range(&self.main_process.as_ref().unwrap(), self.main_process.as_ref().unwrap().get_module_address(MAIN_MODULE).unwrap(), size);
        */

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
    addresses: MemoryAddresses { timr: None },
});


#[no_mangle]
pub extern "C" fn update() {
    LS_CONTROLLER.lock().update();
}
