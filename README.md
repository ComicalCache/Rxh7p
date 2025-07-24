# Rx7h+

A chess engine project named after the last move of IBM Deep Blue in the first match 1996 against the then world champion Garry Kasparov, before he resigned. It marks the first time a computer beat a world champion at chess (remarkable).

The engine is (mostly) UCI capable and can be used in GUIs or hosted on Lichess. For Lichess an appropriate preconfigured config.yml is provided.

## Benchmarking

Benchmarking can be done using cutechess-cli.

```sh
cutechess-cli
  -engine name=new proto=uci cmd=new -engine name=old proto=uci cmd=old -openings file=openings order=random policy=round plies=14 -concurrency 8 -ratinginterval 2 -rounds 500 -games 2 -maxmoves 120 -each st=1 ponder -sprt elo0=0 elo1=10 alpha=0.05 beta=0.05
```

Explaination:

- `-openings file=openings order=random policy=round plies=14`: play a random starting position for 14 plies from the opening book and swap each round.
- `-concurrency 8`: play eight games in parallel.
- `-ratinginterval 2`: print the rating every two finished games.
- `-rounds 500`: play for 500 rounds each two games.
- `-games 2`: play for 500 rounds each two games.
- `-maxmoves 120`: if the game has not ended after 120 moves (240 plies) consider it drawn
- `-each`: apply the following settings to both engines.
- `st=1`: think one second per move.
- `ponder`: allow engines to ponder during opponent moves.
- `sprt elo0=0 elo1=150 alpha=0.05 beta=0.05`: sequential probability ratio test. Hypthoesis H1 is that engine A is stronger than engine B by at least elo0, hypthesis H0 is that engine A is not stronger than B by at least elo1. If either H0 or H1 are fulfilled with error in alpha and beta the match is stopped.

# Bibliography

Pieces of knowledge interesting to anyone reading, myself included.

### Lichess

- https://github.com/lichess-bot-devs/lichess-bot/wiki/Configure-lichess-bot

### Endgame tables

- https://www.talkchess.com/forum/viewtopic.php?t=25311
- https://www.talkchess.com/forum3/viewtopic.php?t=47681

### UCI

<details>

- https://www.chessprogramming.org/UCI
- https://wbec-ridderkerk.nl/html/UCIProtocol.html
- https://official-stockfish.github.io/docs/stockfish-wiki/UCI-&-Commands.html
- https://github.com/official-stockfish/Stockfish/wiki/Terminology

</details>

### Testing the engine

<details>

- https://www.chessprogramming.org/Cutechess-cli
- https://www.reddit.com/r/ComputerChess/comments/m2ertv/comment/gqirufx

</details>

### Transposition Table

<details>

- https://mediocrechess.blogspot.com/2007/01/guide-transposition-tables.html
- https://en.wikipedia.org/wiki/Negamax#Negamax_with_alpha_beta_pruning_and_transposition_tables
- https://www.chessprogramming.org/Principal_Variation

</details>

### Alpha-Beta-Pruning

<details>

- https://www.chessprogramming.org/Alpha-Beta#Negamax_Framework
- https://www.chessprogramming.org/Transposition_Table
- https://www.chessprogramming.org/Iterative_Deepening
- https://www.chessprogramming.org/Principal_Variation_Search
- https://www.chessprogramming.org/Null_Window
- https://en.wikipedia.org/wiki/Principal_variation_search

</details>

### Quiescence Search

<details>

- https://www.chessprogramming.org/Quiescence_Search
- https://www.chessprogramming.org/Horizon_Effect

</details>

### Repetition

<details>

- https://www.chessprogramming.org/Irreversible_Moves
- https://www.chessprogramming.org/Repetitions
- https://en.wikipedia.org/wiki/Threefold_repetition

</details>

### Move Ordering

<details>

- https://www.chessprogramming.org/MVV-LVA
- https://www.chessprogramming.org/Static_Exchange_Evaluation

</details>

### Evaluation

<details>

- https://www.chessprogramming.org/Simplified_Evaluation_Function
- https://www.chessprogramming.org/Piece-Square_Tables
- https://www.chessprogramming.org/PeSTO's_Evaluation_Function
- https://www.chessprogramming.org/Tapered_Eval

</details>
<br>

# TODO

<details>

- https://www.chessprogramming.org/Sequential_Probability_Ratio_Test
- https://web.archive.org/web/20071030220825/http://www.brucemo.com/compchess/programming/pvs.htm
- https://www.chessprogramming.org/Triangular_PV-Table

- https://www.chessprogramming.org/Material#Balance
- https://www.chessprogramming.org/CPW-Engine_recognize
- https://www.chessprogramming.org/Draw_Evaluation
- https://www.chessprogramming.org/Lazy_Evaluation
- https://zwischenzug.substack.com/p/centipawns-suck
- https://www.chessprogramming.org/History_Leaf_Pruning
- https://www.chessprogramming.org/Late_Move_Reductions
- https://www.chessprogramming.org/History_Heuristic
- https://www.chessprogramming.org/Null_Move_Pruning
- Implement endgame tablebases
- https://www.chessprogramming.org/CLOP
- https://www.chessprogramming.org/Lazy_SMP
- https://www.chessprogramming.org/Delta_Pruning
- https://www.chessprogramming.org/Futility_Pruning
- https://www.chessprogramming.org/Backward_Pawn
  - Further reading https://www.stmintz.com/ccc/index.php?id=56328
- https://www.chessprogramming.org/Candidate_Passed_Pawn
- https://www.chessprogramming.org/Pawn_Chain
- https://www.chessprogramming.org/Connected_Pawns
- https://www.chessprogramming.org/Hanging_Pawns
- https://www.chessprogramming.org/Influence_Quantity_of_Pieces
- https://www.chessprogramming.org/Outposts
- https://www.chessprogramming.org/SSE2#SSE2dotproduct (VERY GOOD)
- https://www.chessprogramming.org/Strategic_Test_Suite
- https://www.chessprogramming.org/Trapped_Pieces
- https://www.chessprogramming.org/Bad_Bishop
- https://www.chessprogramming.org/Color_Weakness
- https://www.chessprogramming.org/Returning_Bishop
- https://www.chessprogramming.org/Fianchetto
- https://www.chessprogramming.org/Rook_on_Seventh
- https://www.chessprogramming.org/Tarrasch_Rule
- https://www.chessprogramming.org/Evaluation_Patterns
- https://www.chessprogramming.org/King_Safety
- https://www.chessprogramming.org/Square_Control
- https://www.chessprogramming.org/Center_Control
- https://www.chessprogramming.org/Connectivity
- https://www.chessprogramming.org/Space
- https://www.chessprogramming.org/Tempo
- https://www.chessprogramming.org/Game_Phases
- https://www.chessprogramming.org/Evaluation_Discontinuity
- https://www.chessprogramming.org/Evaluation#Miscellaneous
- https://www.chessprogramming.org/Automated_Tuning
- https://www.chessprogramming.org/Score
- https://www.chessprogramming.org/Point_Value
- https://www.chessprogramming.org/NNUE
- https://www.chessprogramming.org/Evaluation
- https://www.chessprogramming.org/Evaluation_of_Pieces
- https://www.chessprogramming.org/Evaluation_Philosophy (VERY GOOD)
- https://www.chessprogramming.org/Mobility
- https://www.chessprogramming.org/Pawn_Structure
- https://www.chessprogramming.org/Doubled_Pawn
- https://www.chessprogramming.org/Isolated_Pawn
- https://www.chessprogramming.org/Bishop_Pair
- https://www.chessprogramming.org/Rook_on_Open_File
- https://www.chessprogramming.org/Stockfish
- https://www.chessprogramming.org/Stockfish#Search (!!!)
- https://www.chessprogramming.org/Stockfish#Classical_Evaluation (!!!)
- https://www.chessprogramming.org/CPW-Engine_eval
- https://www.chessprogramming.org/Aspiration_Windows

</details>
````
