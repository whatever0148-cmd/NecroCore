; NECRORUNNER-1991: a SUPER-CHIP endless runner on the 128x64 high-resolution screen.
; Key 5 jumps. The ground scrolls with SCL, the runner and the block are 16x16 sprites (DRW n=0),
; and the best score is parked in the RPL flags with LD R, V7.
; Original program written for NecroCore (2026). Not a historical binary.
; Deliberate defects: game_over spins forever, and bonus_level is never called.
; Registers: V2 runner y, V3 jump phase (0 on the ground), V4/V5 block x/y, V7 score,
;            VA runner x, VB scroll parity, VC/VD loop x/y, V0/V1 scratch.
start:
    HIGH
    LD V1, R
    LD VA, 16
    LD V2, 40
    LD V3, 0
    LD V4, 112
    LD V5, 40
    LD V7, 0
    LD VB, 0
    LD I, ground_tile
    LD VC, 0
    LD VD, 56
ground_loop:
    DRW VC, VD, 1
    ADD VC, 8
    SE VC, 128
    JP ground_loop
    LD I, dash_tile
    LD VC, 0
    LD VD, 58
dash_loop:
    DRW VC, VD, 1
    ADD VC, 8
    SE VC, 128
    JP dash_loop
    LD I, runner_sprite
    DRW VA, V2, 0
    LD I, block_sprite
    DRW V4, V5, 0
    CALL toggle_score
loop:
    ; take the old picture off the screen (drawing is XOR)
    LD I, runner_sprite
    DRW VA, V2, 0
    LD I, block_sprite
    DRW V4, V5, 0
    CALL toggle_score
    ; scroll the world 4 pixels left and refill the right edge
    SCL
    LD I, dash_tile
    LD V0, 124
    LD V1, 56
    DRW V0, V1, 1
    LD V0, 1
    XOR VB, V0
    SE VB, 1
    JP no_dash
    LD V0, 124
    LD V1, 58
    DRW V0, V1, 1
no_dash:
    LD V0, 5
    SKNP V0
    CALL start_jump
    SE V3, 0
    CALL fly
    .patch OBST_SPEED pixels the block moves left each frame, 1 to 6 (0xFC is 4)
    ADD V4, 0xFC
    LD V0, 200
    SUB V0, V4
    SNE VF, 0
    CALL respawn
    LD I, block_sprite
    DRW V4, V5, 0
    LD I, runner_sprite
    DRW VA, V2, 0
    SE VF, 0
    JP hit
    CALL toggle_score
    .patch FRAME_DELAY ticks to wait per frame, 2 normal and 1 double speed
    LD V0, 2
    LD DT, V0
wait:
    LD V0, DT
    SE V0, 0
    JP wait
    JP loop
toggle_score:
    LD F, V7
    LD V0, 2
    LD V1, 2
    DRW V0, V1, 5
    RET
start_jump:
    SE V3, 0
    RET
    LD V3, 1
    RET
fly:
    ADD V3, 1
    LD V0, 9
    SUB V0, V3
    SE VF, 1
    JP falling
    .patch JUMP_STEP pixels gained per frame while rising (0xFD is 3 up)
    ADD V2, 0xFD
    RET
falling:
    ADD V2, 3
    SE V3, 17
    RET
    LD V3, 0
    LD V2, 40
    RET
respawn:
    LD V4, 112
    ADD V7, 1
    RET
hit:
    LD R, V7
    LD V6, 0
shake:
    SCD 2
    SCR
    ADD V6, 1
    LD V0, 3
    LD DT, V0
shake_wait:
    LD V0, DT
    SE V0, 0
    JP shake_wait
    SE V6, 8
    JP shake
game_over:
    JP game_over
bonus_level:
    ; Cut from the 1991 release. Nothing calls this any more.
    LOW
    LD V0, 1
    RET
ground_tile:
    DB 0xFF, 0x00
dash_tile:
    DB 0xF0, 0x00
runner_sprite:
    DB 0x03, 0xC0, 0x07, 0xE0, 0x07, 0xE0, 0x03, 0xC0
    DB 0x01, 0x80, 0x0F, 0xF0, 0x1F, 0xF8, 0x3D, 0xBC
    DB 0x39, 0x9C, 0x01, 0x80, 0x03, 0xC0, 0x07, 0xE0
    DB 0x06, 0x60, 0x0E, 0x70, 0x0C, 0x30, 0x1C, 0x38
block_sprite:
    DB 0xFF, 0xFF, 0x80, 0x01, 0xBF, 0xFD, 0xA0, 0x05
    DB 0xAF, 0xF5, 0xA8, 0x15, 0xAB, 0xD5, 0xAA, 0x55
    DB 0xAA, 0x55, 0xAB, 0xD5, 0xA8, 0x15, 0xAF, 0xF5
    DB 0xA0, 0x05, 0xBF, 0xFD, 0x80, 0x01, 0xFF, 0xFF
