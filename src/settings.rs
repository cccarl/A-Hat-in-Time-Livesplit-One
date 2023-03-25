#[derive(asr::Settings)]
pub struct Settings {
    #[default = false]
    /// IL Mode
    pub il_mode: bool,
    #[default = true]
    /// Start (Global)
    pub start: bool,
    #[default = true]
    /// Start only on empty files
    pub start_new_file: bool,
    #[default = true]
    /// Reset (Global)
    pub reset: bool,
    #[default = true]
    /// Split (Global)
    pub split: bool,

    #[default = true]
    /// Split - Simple - New Time Piece
    pub split_simple_new_tp: bool,
    #[default = true]
    /// Split - Simple - Any Time Piece
    pub split_simple_any_tp: bool,
    #[default = false]
    /// Split - Simple - Act Entry
    pub split_simple_act_entry: bool,
    #[default = false]
    /// Split - Simple - Death Wish Back To Hub
    pub split_simple_dw_bth: bool,
    #[default = false]
    /// Split - Simple - Yarn
    pub splits_simple_yarn: bool,
    #[default = false]
    /// Split - Simple - Checkpoint
    pub splits_simple_cp: bool,

    #[default = false]
    /// Split - Detailed - Mafia Town - Welcome to Mafia Town Fight Start
    pub splits_detailed_cp_1_1_1: bool,
    #[default = false]
    /// Split - Detailed - Mafia Town - Welcome to Mafia Town Time Piece
    pub splits_detailed_tp_1_1: bool,

    #[default = false]
    /// Split - Detailed - Mafia Town - Barrel Battle Fight Start
    pub splits_detailed_cp_1_2_1: bool,
    #[default = false]
    /// Split - Detailed - Mafia Town - Barrel Battle Time Piece
    pub splits_detailed_tp_1_2: bool,

    #[default = false]
    /// Split - Detailed - Mafia Town - She Came From Outer Space Scare
    pub splits_detailed_cp_1_3_1: bool,
    #[default = false]
    /// Split - Detailed - Mafia Town - She Came From Outer Space Time Piece
    pub splits_detailed_tp_1_3: bool,

    #[default = false]
    /// Split - Detailed - Mafia Town - Down With The Mafia! Vent
    pub splits_detailed_cp_1_4_1: bool,
    #[default = false]
    /// Split - Detailed - Mafia Town - Down With The Mafia! Boss Start
    pub splits_detailed_cp_1_4_2: bool,
    #[default = false]
    /// Split - Detailed - Mafia Town - Down With The Mafia! Time Piece
    pub splits_detailed_tp_1_4: bool,

    #[default = false]
    /// Split - Detailed - Mafia Town - Cheating The Race Time Piece
    pub splits_detailed_tp_1_5: bool,
    #[default = false]
    /// Split - Detailed - Mafia Town - Heating Up Mafia Town Time Piece
    pub splits_detailed_tp_1_6: bool,
    #[default = false]
    /// Split - Detailed - Mafia Town - The Golden Vault Time Piece
    pub splits_detailed_tp_1_7: bool,

    #[default = false]
    /// Split - Detailed - Battle of the Birds - Dead Bird Studio Warp Exploit
    pub splits_detailed_cp_2_1_5: bool,
    #[default = false]
    /// Split - Detailed - Battle of the Birds - Dead Bird Studio Time Piece
    pub splits_detailed_tp_2_1: bool,
    #[default = false]
    /// Split - Detailed - Battle of the Birds - Murder on the Owl Express Time Piece
    pub splits_detailed_tp_2_2: bool,
    #[default = false]
    /// Split - Detailed - Battle of the Birds - Picture Perfect Time Piece
    pub splits_detailed_tp_2_3: bool,
    #[default = false]
    /// Split - Detailed - Battle of the Birds - Train Rush Time Piece
    pub splits_detailed_tp_2_4: bool,
    #[default = false]
    /// Split - Detailed - Battle of the Birds - The Big Parade Time Piece
    pub splits_detailed_tp_2_5: bool,
    #[default = false]
    /// Split - Detailed - Battle of the Birds - Award Ceremony Time Piece
    pub splits_detailed_tp_2_6: bool,

    #[default = false]
    /// Split - Detailed - Subcon Forest - Contractual Obligations Time Piece
    pub splits_detailed_tp_3_1: bool,
    #[default = false]
    /// Split - Detailed - Subcon Forest - The Subcon Well Time Piece
    pub splits_detailed_tp_3_2: bool,
    #[default = false]
    /// Split - Detailed - Subcon Forest - Toilet of Doom Time Piece
    pub splits_detailed_tp_3_3: bool,
    #[default = false]
    /// Split - Detailed - Subcon Forest - Queen Vanessa's Manor Time Piece
    pub splits_detailed_tp_3_4: bool,
    #[default = false]
    /// Split - Detailed - Subcon Forest - Mail Delivery Service Time Piece
    pub splits_detailed_tp_3_5: bool,
    #[default = false]
    /// Split - Detailed - Subcon Forest - Your Contract Has Expired Time Piece
    pub splits_detailed_tp_3_6: bool,
    #[default = false]
    /// Split - Detailed - Alpine Skyline Time Pieces
    pub splits_detailed_tp_4: bool,
    #[default = false]
    /// Split - Detailed - Finale Time Pieces
    pub splits_detailed_tp_5: bool,
    #[default = false]
    /// Split - Detailed - The Artic Cruise - Bon Voyage! Time Piece
    pub splits_detailed_tp_6_1: bool,
    #[default = false]
    /// Split - Detailed - The Artic Cruise - Ship Shape Time Piece
    pub splits_detailed_tp_6_2: bool,
    #[default = false]
    /// Split - Detailed - The Artic Cruise- Rock The Boat Time Piece
    pub splits_detailed_tp_6_3: bool,
    #[default = false]
    /// Split - Detailed - Nyakuza Time Pieces Time Pieces
    pub splits_detailed_tp_7: bool,
}
