/*  
TODO: 
split (simple)
start
reset
asl helper functions, in new file?
settings
detailed splits monkas
*/
mod memory;
mod settings;
mod splits;

use settings::Settings;
use std::collections::HashMap;
use asr::{Process, watcher::Pair, Address};
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
    Dlc,
    Cdlc,
    Modding,
    Release,
}

#[derive(Default)]
struct MemoryAddresses {
    timr: Option<Address>, // gives access to main timer values
    save_data_base: Option<Address>, // base of the pointer path that leads to save data
    save_data_offsets: HashMap<String, [u64; 2]>,
    position_x_pointer_path: Vec<u64>,
    position_y_pointer_path: Vec<u64>,
    position_z_pointer_path: Vec<u64>,
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
    just_got_time_piece: Pair<i32>, // green act timer -> 1, otherwise 0
    game_time: Pair<f64>, // igt in loading screen
    act_time: Pair<f64>,
    real_game_time: Pair<f64>, // igt in HUD
    real_act_time: Pair<f64>,
    tp_count: Pair<i32>,
    // save data values
    yarn: Pair<i32>,
    chapter: Pair<i32>,
    act: Pair<i32>,
    checkpoint: Pair<i32>,
    // position values
    x: Pair<f32>,
    y: Pair<f32>,
    z: Pair<f32>,
}

struct State {
    started_up: bool,
    settings: Option<Settings>,
    main_process: Option<Process>,
    values: Lazy<MemoryValues>,
    addresses: Lazy<MemoryAddresses>,
    patch_type: PatchType,
}

impl State {

    fn startup(&mut self) {

        self.settings = Some(settings::Settings::register());
        asr::set_tick_rate(10.0);
        self.started_up = true;
    }

    fn init(&mut self) -> Result<(), &str> {
        self.hat_sig_scan_start()?;
        asr::set_tick_rate(120.0);
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
                    asr::print_message(&format!("ERROR: init() didn't finish properly, message: {message}"));
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
            self.patch_type = PatchType::Unknown;
            return;
        }

        if self.refresh_mem_values().is_err() {
            return;
        }

        // unwrap settings
        let Some(settings) = &self.settings else { return };

        // LS controller logic section

        // start when opening a file depending on the fresh file setting, or entering a level in IL mode
        if settings.start
        && (((self.values.tp_count.current < 1 || !settings.start_new_file) && self.values.timer_state.old == 0 && self.values.timer_state.current == 1)
        || (settings.il_mode && self.values.act_timer_is_visible.increased())) {
            asr::timer::start();
        }

        // reset, always when going to main menu with an orange running main timer (state 1), and when restarting a level/going to the hub in IL mode
        if settings.reset
        && ((self.values.timer_state.current == 0 && self.values.timer_state.old == 1) 
        || (settings.il_mode && (self.values.act_timer_is_visible.decreased() || (self.values.real_act_time.decreased() && self.values.real_act_time.current == 0.0)))){
            asr::timer::reset();
        }

        // game time set
        asr::timer::pause_game_time();
        if settings.il_mode {
            asr::timer::set_game_time(asr::time::Duration::seconds_f64(self.values.real_act_time.current));
        } else {
            asr::timer::set_game_time(asr::time::Duration::seconds_f64(self.values.real_game_time.current));
        }

        // splits
        if self.should_split() && settings.split {
            asr::timer::split();
        }
        

    }

}

static LS_CONTROLLER: Spinlock<State> = const_spinlock(State {
    started_up: false,
    settings: None,
    main_process: None,
    values: Lazy::new(Default::default),
    addresses: Lazy::new(Default::default),
    patch_type: PatchType::Unknown,
});


#[no_mangle]
pub extern "C" fn update() {
    LS_CONTROLLER.lock().update();
}
