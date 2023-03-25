use crate::State;

impl State {

    pub fn should_split(&self) -> bool {

        let Some(settings) = &self.settings else { return false };

        let mut split;

        // simple/generic splits
        split = 
            // generic tp split
            (self.values.tp_count.current == self.values.tp_count.old + 1 && settings.split_simple_new_tp)
            // any time piece that stops the act timer
            || (self.values.just_got_time_piece.increased() && settings.split_simple_any_tp)
            // act entry, TODO: when current rift is saved, add condition of "none"
            || (self.values.act_timer_is_visible.increased() && settings.split_simple_act_entry && !settings.il_mode)
            // TODO: dw time piece
            // dw back to hub, TODO: avoid splitting when there also was a recent "any tp" split (asl did this with a timer)
            || (self.values.chapter.changed_from(&97) && settings.split_simple_dw_bth)
            // yarn grab
            || (self.values.yarn.current == self.values.yarn.old + 1 && settings.splits_simple_yarn)
            // checkpoint
            || (self.values.checkpoint.changed() && self.values.checkpoint.current != 0 && settings.splits_simple_cp);

        // detailed splits

        // new time piece per chapter + act
        // TODO: check if it would be a good idea to make it for "just got time piece" increased too in the if
        if self.values.tp_count.increased() && !split {
            split = match (self.values.chapter.current, self.values.checkpoint.current) {
                (1, 1) => settings.splits_detailed_tp_1_1,
                (1, 2) => settings.splits_detailed_tp_1_2,
                (1, 3) => settings.splits_detailed_tp_1_3,
                (1, 4) => settings.splits_detailed_tp_1_4,
                (1, 5) => settings.splits_detailed_tp_1_5,
                (1, 6) => settings.splits_detailed_tp_1_6,
                (1, 7) => settings.splits_detailed_tp_1_7,
                (2, 1) => settings.splits_detailed_tp_2_1,
                (2, 2) => settings.splits_detailed_tp_2_2,
                (2, 3) => settings.splits_detailed_tp_2_3,
                (2, 4) => settings.splits_detailed_tp_2_4,
                (2, 5) => settings.splits_detailed_tp_2_5,
                (2, 6) => settings.splits_detailed_tp_2_6,
                (3, 1) => settings.splits_detailed_tp_3_1,
                (3, 2) => settings.splits_detailed_tp_3_2,
                (3, 3) => settings.splits_detailed_tp_3_3,
                (3, 4) => settings.splits_detailed_tp_3_4,
                (3, 5) => settings.splits_detailed_tp_3_5,
                (3, 6) => settings.splits_detailed_tp_3_6,
                (4, _) => settings.splits_detailed_tp_4,
                (5, _) => settings.splits_detailed_tp_5,
                (6, 1) => settings.splits_detailed_tp_6_1,
                (6, 2) => settings.splits_detailed_tp_6_2,
                (6, 3) => settings.splits_detailed_tp_6_3,
                (7, _) => settings.splits_detailed_tp_7,
                _ => false,
            }
        }

        // checkpoint splits
        if self.values.checkpoint.changed() && !split {
            split = match (self.values.chapter.current, self.values.checkpoint.current, self.values.checkpoint.current) {
                (1, 1, 1) => settings.splits_detailed_cp_1_1_1,
                (1, 2, 1) => settings.splits_detailed_cp_1_2_1,
                (1, 3, 1) => settings.splits_detailed_cp_1_3_1,
                (1, 4, 1) => settings.splits_detailed_cp_1_4_1,
                (1, 4, 2) => settings.splits_detailed_cp_1_4_2,
                (2, 1, 5) => settings.splits_detailed_cp_2_1_5,
                // TODO: the rest lol
                _ => false,
            }
        }

        // TODO: tp delayed split

        // TODO: checkpoint + pause splits

        // TODO: position splits... will i do the same thing about having a primary volume unlocking the split?

        // TODO: time rift splits, detect current split, will i use the positions again?? can i get the map name in mem instead?

        // TODO: purple rift checkpoint splits

        split
    }
}
