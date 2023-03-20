use crate::State;

impl State {
    pub fn should_split(&self) -> bool {
        // simple/generic splits
        // generic tp split
        (self.values.tp_count.current == self.values.tp_count.old + 1 && self.settings["split_simple_new_tp"])
        // any time piece that stops the act timer
        || (self.values.just_got_time_piece.increased() && self.settings["split_simple_any_tp"])
        // act entry, TODO: when current rift is saved, add condition of "none"
        || (self.values.act_timer_is_visible.increased() && self.settings["split_simple_act_entry"] && !self.settings["il_mode"])
        // TODO: dw time piece
        // dw back to hub, TODO: avoid splitting when there also was a recent "any tp" split (asl did this with a timer)
        || (self.values.chapter.changed_from(&97) && self.settings["split_simple_dw_bth"])
        // yarn grab
        || (self.values.yarn.current == self.values.yarn.old + 1 && self.settings["splits_simple_yarn"])
        // checkpoint
        || (self.values.checkpoint.changed() && self.values.checkpoint.current == 0 && self.settings["splits_simple_cp"])
        // detailed splits
    }
}
