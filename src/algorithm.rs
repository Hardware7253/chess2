use crate::board::board_representation;
use crate::board::move_generator::EnemyAttacks;
use crate::TeamBitboards;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Move {
    pub initial_piece_coordinates: board_representation::BoardCoordinates,
    pub final_piece_bit: usize,
    pub value: i8,
    pub heatmap_value: u16,
}

impl Move {
    pub fn new() -> Self {
        Move {
            initial_piece_coordinates: board_representation::BoardCoordinates {
                board_index: 0,
                bit: 0,
            },
            final_piece_bit: 0,
            value: 0,
            heatmap_value: 0,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct MinMax {
    max_move: Option<Move>,
    min_value: Option<i8>,
}

// Update MinMax struct if new move has a value lesser or greater than min/max fields
// Initialize MinMax if it hasn't been allready
fn update_min_max(piece_move: Move, mut min_max: MinMax) -> MinMax {
    match min_max.max_move {
        Some(_) => (),
        None => {

            // If min_max has not yet been initialized then initialize it with piece_move
            return MinMax {
                max_move: Some(piece_move),
                min_value: Some(piece_move.value),
            };
        },
    }

    let max_value = min_max.max_move.unwrap().value;
    let min_value = min_max.min_value.unwrap();

    if piece_move.value > max_value {
        min_max.max_move = Some(piece_move);
    } else if piece_move.value < min_value {
        min_max.min_value = Some(piece_move.value);
    }

    min_max
}

fn update_prune_value(master_team: bool, min_max: &MinMax) -> Option<i8> {
    if master_team {
        match min_max.max_move {
            Some(max_move) => return Some(max_move.value),
            None => return None,
        }
    } else {
        return min_max.min_value;
    }
}

pub fn gen_best_move(master_team: bool, search_depth: usize, current_depth: usize, init_value: i8, parent_value: Option<i8>, opening_heatmap: &[[u16; 64]; 12], board: board_representation::Board, pieces_info: &[crate::piece::constants::PieceInfo; 12]) -> Move {
    use crate::board::move_generator;
    use crate::board::move_generator::TurnError;

    let mut empty_move = Move::new();

    // If current depth and search depth are equal stop searching down the move tree
    if current_depth == search_depth {
        empty_move.value = init_value;
        return empty_move;
    }

    // Get friendly and enemy team BoardCoordinates
    let friendly_king_index;
    let enemy_king_index;
    if board.whites_move {
        friendly_king_index = 5;
        enemy_king_index = 11;
    } else {
        friendly_king_index = 11;
        enemy_king_index = 5;
    }

    let friendly_king = board_representation::BoardCoordinates {
        board_index: friendly_king_index,
        bit: crate::find_bit_on(board.board[friendly_king_index], 0),
    };

    let enemy_king = board_representation::BoardCoordinates {
        board_index: enemy_king_index,
        bit: crate::find_bit_on(board.board[enemy_king_index], 0),
    };
    
    // Generate team bitboards
    let team_bitboards = TeamBitboards::new(friendly_king_index, &board);

    // Generate enemy attacks
    let enemy_attacks = move_generator::gen_enemy_attacks(&friendly_king, team_bitboards, &board, pieces_info);

    // Generate moves
    let moves = &order_moves(true, &board, &enemy_attacks, &friendly_king, opening_heatmap, team_bitboards, pieces_info);

    let mut min_max = MinMax {
        max_move: None,
        min_value: None,
    };

    let mut prune_value: Option<i8> = None;

    for i in 0..moves.len() {
        let initial_piece_coordinates = moves[i].initial_piece_coordinates;
        let final_piece_bit = moves[i].final_piece_bit;

        let new_turn_board = move_generator::new_turn(&initial_piece_coordinates, final_piece_bit, friendly_king, &enemy_king, &enemy_attacks, team_bitboards, board, &pieces_info);
        
        match new_turn_board {

            // Only continue searching down the move tree if the move didn't result in an invalid move or the end of the game
            Ok(new_board) => {
                let mut move_value = new_board.points_delta;
                
                // If the current branch is not the master team then it's move values are negative (because they negatively impact the master team)
                if !master_team {
                    move_value *= -1;
                }

                let branch_value = init_value + move_value;

                let piece_move = gen_best_move(!master_team, search_depth, current_depth + 1, branch_value, prune_value, opening_heatmap, new_board, pieces_info);
                let piece_move = Move {
                    initial_piece_coordinates: initial_piece_coordinates,
                    final_piece_bit: final_piece_bit,
                    value: piece_move.value,
                    heatmap_value: 0,
                };
                
                min_max = update_min_max(piece_move, min_max);
                prune_value = update_prune_value(master_team, &min_max);
            },
            Err(error) => {

                // Update min_max with value of game ending if the game ended
                let mut branch_value;
                let valid_move;

                match error {
                    TurnError::Win => {branch_value = 127; valid_move = true},
                    TurnError::Draw => {branch_value = 0; valid_move = true},
                    TurnError::InvalidMove => {branch_value = 0; valid_move = false},
                    TurnError::InvalidMoveCheck => {branch_value = 0; valid_move = false},
                }

                // If the current branch is not the master team then it's move values are negative (because they negatively impact the master team)
                if !master_team {
                    branch_value *= -1;
                }

                if valid_move {
                    let piece_move = Move {
                        initial_piece_coordinates: initial_piece_coordinates,
                        final_piece_bit: final_piece_bit,
                        value: branch_value,
                        heatmap_value: 0,
                    };

                    min_max = update_min_max(piece_move, min_max);
                    prune_value = update_prune_value(master_team, &min_max);
                }

                continue;
            },
        }

        // Alpha beta pruning
        match parent_value {
            Some(value) => {
                if master_team {
                    match min_max.max_move {
                        Some(max_move) => {
                            if max_move.value >= value {
                                break;
                            }
                        },
                        None => (),
                    }
                } else {
                    match min_max.min_value {
                        Some(min_value) => {
                            if min_value <= value {
                                break;
                            }
                        },
                        None => (),
                    }
                }
            },
            None => ()
        }
    }

    // Return min/max values depending on the team
    if master_team {
        return min_max.max_move.unwrap();
    } else {
        empty_move.value = min_max.min_value.unwrap();
        return empty_move;
    }
}

// Returns a vec with potential moves
// If sort is true the moves will be ordered from best to worst
// All moves are valid apart from king moves
fn order_moves(sort: bool, board: &board_representation::Board, enemy_attacks: &EnemyAttacks, friendly_king: &board_representation::BoardCoordinates, opening_heatmap: &[[u16; 64]; 12], team_bitboards: crate::TeamBitboards, pieces_info: &[crate::piece::constants::PieceInfo; 12]) -> Vec<Move> {
    use crate::bit_on;
    
    let mut moves: Vec<Move> = Vec::new();

    // Get friendly and enemy board indexes
    let friendly_indexes;
    let enemy_index_bottom; // Inclusive
    let enemy_index_top; // Not inclusive
    if board.whites_move {
        friendly_indexes = 0..6;
        enemy_index_bottom = 6;
        enemy_index_top = 12;
    } else {
        friendly_indexes = 6..12;
        enemy_index_bottom = 0;
        enemy_index_top = 6;
    }

    for i in friendly_indexes {
        let piece_value = pieces_info[i].value;

        for initial_bit in 0..64 {
            let initial_piece_coordinates = board_representation::BoardCoordinates {
                board_index: i,
                bit: initial_bit,
            };

            // If there is no piece on the board at this bit got to the next bit
            if !bit_on(board.board[i], initial_bit) {
                continue;
            }

            let piece_moves = crate::board::move_generator::gen_piece(&initial_piece_coordinates, None, team_bitboards, false, board, pieces_info);
            
            for final_bit in 0..64 {
                let heatmap_value = opening_heatmap[i][final_bit];

                // Check the piece can move to final_bit or piece is a king
                // Because this function does not account for castling those moves cannot be ruled out for the king
                if bit_on(piece_moves.moves_bitboard, final_bit) {
                    
                    // Get value of move based on value of captured piece
                    let mut move_value = 0;
                    if bit_on(team_bitboards.enemy_team, final_bit) { // If an enemy piece is in the same bit as the friendly pieces final_bit then it has been captured

                        for j in enemy_index_bottom..enemy_index_top {
                            if bit_on(board.board[j], final_bit) {
                                let capture_value = pieces_info[j].value;
    
                                // If an enemy can move to the captured square there will likely be a trade
                                if bit_on(enemy_attacks.enemy_attack_bitboard, final_bit) {
                                    move_value = piece_value - capture_value;
                                } else { // If an enemy can't move to the captured square then the friendly team gets the entire value of the captured piece
                                    move_value = capture_value;
                                }
    
                                // Once the piece that has been captured is found break the loop
                                break;
                            }
                        }
                    }

                    // Push move to moves vec
                    moves.push(Move {
                        initial_piece_coordinates: initial_piece_coordinates,
                        final_piece_bit: final_bit,
                        value: move_value,
                        heatmap_value: heatmap_value,
                    });
                } else if &initial_piece_coordinates == friendly_king { // Add potentially invalid king moves to moves vec to account for castling
                    moves.push(Move {
                        initial_piece_coordinates: initial_piece_coordinates,
                        final_piece_bit: final_bit,
                        value: 0,
                        heatmap_value: heatmap_value,
                    });
                }
            }
        }
    }

    // Sort moves and return
    if sort {

        // Sort moves by value first
        // Sort moves by heatmap_value if they have the same value
        // https://stackoverflow.com/questions/70193935/how-to-sort-a-vec-of-structs-by-2-or-multiple-fields
        moves.sort_by(| a, b | if a.value == b.value {
            b.heatmap_value.partial_cmp(&a.heatmap_value).unwrap()
        } else {
            b.value.partial_cmp(&a.value).unwrap()
        });
    }
    moves
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_moves_test() {
        use crate::board::board_representation;
        use crate::board::move_generator;

        let opening_heatmap = [[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 10, 1, 18, 10, 9, 9, 1, 0, 1, 33, 61, 475, 338, 22, 6, 5, 51, 142, 1144, 2288, 2246, 392, 88, 80, 88, 74, 361, 111, 276, 124, 322, 62, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 1, 0, 4, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 35, 32, 94, 499, 3, 0], [1, 0, 0, 0, 0, 0, 0, 2, 0, 0, 2, 0, 0, 19, 0, 2, 0, 0, 15, 1, 2, 7, 0, 0, 1, 31, 0, 19, 145, 2, 79, 0, 9, 0, 11, 268, 58, 0, 1, 7, 16, 17, 1470, 1, 3, 2054, 9, 15, 0, 0, 2, 115, 62, 1, 0, 0, 0, 1, 0, 0, 5, 2, 2, 0], [1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 3, 20, 22, 1, 0, 0, 1, 35, 0, 0, 17, 0, 2, 0, 314, 1, 13, 2, 0, 292, 0, 139, 2, 509, 2, 0, 47, 0, 35, 6, 108, 1, 162, 124, 1, 2, 3, 0, 51, 19, 57, 148, 1, 205, 0, 1, 0, 2, 0, 0, 3, 0, 0], [0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 3, 2, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 4, 1, 2, 0, 24, 22, 0, 13, 32, 3, 2, 24, 3, 0, 48, 7, 17, 6, 42, 0, 0, 0, 0, 66, 49, 67, 3, 0, 0, 0, 1, 0, 3, 3, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 9, 4, 1, 0, 0, 0, 23, 4, 0, 26, 498, 6], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 348, 125, 418, 716, 867, 40, 525, 86, 17, 238, 834, 1360, 1326, 216, 134, 18, 0, 13, 174, 512, 190, 170, 68, 4, 1, 0, 34, 3, 4, 37, 4, 0, 0, 6, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0], [0, 8, 3, 3, 17, 458, 5, 0, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0], [0, 13, 0, 2, 1, 1, 8, 0, 0, 4, 3, 219, 58, 2, 1, 0, 21, 32, 1057, 15, 1, 1874, 4, 29, 56, 0, 8, 130, 31, 3, 1, 10, 0, 9, 4, 40, 190, 2, 21, 0, 0, 0, 31, 0, 2, 1, 3, 0, 0, 0, 1, 2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0], [0, 0, 0, 0, 1, 3, 0, 1, 1, 74, 0, 44, 307, 0, 387, 2, 20, 31, 2, 44, 56, 5, 9, 5, 27, 0, 241, 0, 2, 79, 3, 1, 0, 297, 3, 5, 2, 0, 98, 4, 0, 0, 60, 3, 1, 8, 0, 3, 0, 0, 0, 5, 1, 3, 1, 1, 0, 1, 0, 1, 0, 3, 0, 0], [1, 1, 2, 4, 5, 0, 0, 0, 0, 0, 36, 10, 62, 0, 0, 0, 0, 28, 0, 10, 2, 36, 6, 0, 79, 0, 0, 53, 5, 4, 12, 2, 0, 1, 1, 9, 3, 2, 0, 51, 1, 0, 1, 0, 0, 0, 0, 2, 0, 2, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 2, 7, 0, 4, 458, 0, 0, 0, 0, 0, 5, 17, 2, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]];
        
        let board = board_representation::fen_decode("k7/8/8/8/4r3/3P4/8/7K w - - 0 1", true);

        let king = board_representation::BoardCoordinates {
            board_index: 5,
            bit: 63,
        };

        let team_bitboards = TeamBitboards::new(king.board_index, &board);

        let pieces_info = crate::piece::constants::gen();

        let enemy_attacks = move_generator::gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

        let result = order_moves(true, &board, &enemy_attacks, &king, &opening_heatmap, team_bitboards, &pieces_info);

        let best_move = Move {
            initial_piece_coordinates: board_representation::BoardCoordinates {
                board_index: 0,
                bit: 43,
            },
            final_piece_bit: 36,
            value: 5,
            heatmap_value: 2246,
        };

        assert_eq!(result[0], best_move);
    }

    #[test]
    fn update_min_max_test() {
        use crate::board::board_representation;

        let max_move = Move {
            initial_piece_coordinates: board_representation::BoardCoordinates {
                board_index: 0,
                bit: 43,
            },
            final_piece_bit: 36,
            value: 3,
            heatmap_value: 0,
        };

        let piece_move = Move {
            initial_piece_coordinates: board_representation::BoardCoordinates {
                board_index: 0,
                bit: 0,
            },
            final_piece_bit: 0,
            value: 5,
            heatmap_value: 0,
        };

        let min_max = MinMax {
            max_move: None,
            min_value: None,
        };

        let min_max = update_min_max(max_move, min_max);
        let min_max = update_min_max(piece_move, min_max);

        let expected = MinMax {
            max_move: Some(piece_move),
            min_value: Some(3),
        };

        assert_eq!(min_max, expected);
    }

    #[test]
    fn update_prune_value_test() {
        use crate::board::board_representation;

        let piece_move = Move {
            initial_piece_coordinates: board_representation::BoardCoordinates {
                board_index: 0,
                bit: 0,
            },
            final_piece_bit: 0,
            value: 5,
            heatmap_value: 0,
        };

        let min_max = MinMax {
            max_move: Some(piece_move),
            min_value: Some(3),
        };

        let result = update_prune_value(true, &min_max);

        assert_eq!(result, Some(5));
    }

    #[test]
    fn gen_best_move_test1() {
        use crate::board::board_representation;

        let board = board_representation::fen_decode("7k/2K5/8/8/8/r2r4/3R3n/8 w - - 0 1", true);

        let pieces_info = crate::piece::constants::gen();
        
        let result = gen_best_move(true, 3, 0, 0, None, &[[0u16; 64]; 12], board, &pieces_info);

        let expected = Move {
            initial_piece_coordinates: board_representation::BoardCoordinates {
                board_index: 1,
                bit: 51,
            },
            final_piece_bit: 55,
            value: 3,
            heatmap_value: 0,
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn gen_best_move_test2() { // Test a capture with en passant being the best move
        use crate::board::board_representation;

        let board = board_representation::fen_decode("K7/8/8/4pP2/8/8/8/k7 w - e6 0 1", true);

        let pieces_info = crate::piece::constants::gen();
        
        let result = gen_best_move(true, 3, 0, 0, None, &[[0u16; 64]; 12], board, &pieces_info);

        let expected = Move {
            initial_piece_coordinates: board_representation::BoardCoordinates {
                board_index: 0,
                bit: 29,
            },
            final_piece_bit: 20,
            value: 1,
            heatmap_value: 0,
        };

        assert_eq!(result, expected);
    }

    /*
    #[test]
    fn gen_best_move_test3() {
        use crate::board::board_representation;

        let board = board_representation::fen_decode("1nb1kb1r/8/2p3p1/1p1pP2p/7P/2P3Pn/4Bq1N/Q2K4 b - - 0 1", true);

        let pieces_info = crate::piece::constants::gen();
        
        let result = gen_best_move(true, 6, 0, 0, None, &[[0u16; 64]; 12], board, &pieces_info);

        let expected = Move {
            initial_piece_coordinates: board_representation::BoardCoordinates {
                board_index: 0,
                bit: 29,
            },
            final_piece_bit: 20,
            value: 1,
            heatmap_value: 0,
        };

        assert_eq!(result, expected);
    }
    */
}