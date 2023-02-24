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
    // main timer vars
    timer_state: Pair<i32>, // 0 for inactive, 1 for orange running, 2 for green stopped
    unpause_time: Pair<f64>,
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

    fn init(&mut self) {
        // idk just some random ass number
        let size = 0x3200000;

        asr::set_tick_rate(120.0);

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

        let scan_result_address = TIMR_AOB.scan_process_range(process, hat_main_add, size).unwrap();
        self.addresses.timr = Some(Address(scan_result_address.0 - hat_main_add.0));

        /*
        const SAVE_DATA_AOB_VACU: Signature<21> = Signature::new("48 8B 1D ?? ?? ?? ?? 48 85 DB 74 ?? 48 8B 5B ?? 48 85 DB 74 ??");
        SAVE_DATA_AOB_VACU.scan_process_range(&self.main_process.as_ref().unwrap(), self.main_process.as_ref().unwrap().get_module_address(MAIN_MODULE).unwrap(), size);
        */


    }

    fn refresh_mem_values(&mut self) -> Result<&str, &str> {

        let main_module_addr = match &self.main_process {
            Some(info) => match info.get_module_address(MAIN_MODULE) {
                Ok(address) => address,
                Err(_) => return Err("Could not get module address when refreshing memory values.")
            },
            None => return Err("Process info is not initialized.")
        };

        let process = self.main_process.as_ref().unwrap();

        // insert read int calls here
        if let Ok(value) = process.read_pointer_path64::<f64>(main_module_addr.0, &[self.addresses.timr.unwrap().0 + 0x34]) {
            update_pair("Game Time", value, &mut self.values.real_game_time);
        };

        Ok("Success")
    }

    fn update(&mut self) {

        if !self.started_up {
            self.startup();
        }

        if self.main_process.is_none() {
            self.main_process = Process::attach(MAIN_MODULE);
            if self.main_process.is_some() {
                self.init();
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
