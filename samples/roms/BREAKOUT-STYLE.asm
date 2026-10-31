; BREAKOUT-STYLE: a paddle, a ball and two rows of eight bricks.
; Keys: 4 / 6 move the paddle. Clear every brick to win, miss the ball to lose.
; Original program written for NecroCore (2026). Not a historical binary.
; Registers: V2 paddle x, V4/V5 ball x/y, V6/V7 ball dx/dy, V8 row 0 bricks,
;            V9 row 1 bricks, VA gravity, VB frame counter, VC scratch, V0/V1 scratch.
start:
    CLS
    LD V2, 28
    LD V8, 0xFF
    LD V9, 0xFF
    LD VB, 0
    .patch GRAVITY added to dy every 16 frames (0 off, 1 pulls down)
    LD VA, 0
    CALL reset_ball
loop:
    CLS
    LD V0, 4
    SKNP V0
    ADD V2, 0xFE
    LD V0, 6
    SKNP V0
    ADD V2, 2
    SNE V2, 0xFE
    LD V2, 0
    SNE V2, 58
    LD V2, 56
    ; gravity every 16 frames
    ADD VB, 1
    SNE VB, 16
    CALL apply_gravity
    ; move the ball
    ADD V4, V6
    ADD V5, V7
    SNE V4, 0
    LD V6, 1
    SNE V4, 63
    LD V6, 0xFF
    SNE V5, 0
    LD V7, 1
    ; brick rows: row 0 at y 2..4, row 1 at y 6..8
    SNE V5, 5
    CALL hit_row0
    SNE V5, 9
    CALL hit_row1
    ; paddle at y 30, 8 wide
    SNE V5, 29
    CALL hit_paddle
    SNE V5, 31
    JP lose
    ; draw
    LD I, paddle_sprite
    LD V0, 30
    DRW V2, V0, 1
    LD I, ball_sprite
    DRW V4, V5, 1
    LD V0, 2
    LD V1, V8
    CALL draw_row
    LD V0, 6
    LD V1, V9
    CALL draw_row
    LD V0, V8
    OR V0, V9
    SNE V0, 0
    JP win
    LD V0, 1
    LD DT, V0
wait:
    LD V0, DT
    SE V0, 0
    JP wait
    JP loop
apply_gravity:
    LD VB, 0
    ADD V7, VA
    RET
hit_paddle:
    LD V0, V4
    SUB V0, V2
    SE VF, 1
    RET
    LD V1, 7
    SUB V1, V0
    SE VF, 1
    RET
    LD V7, 0xFF
    RET
hit_row0:
    CALL column_bit
    LD V1, V0
    AND V1, V8
    SE V1, 0
    XOR V8, V0
    SE V1, 0
    LD V7, 1
    RET
hit_row1:
    CALL column_bit
    LD V1, V0
    AND V1, V9
    SE V1, 0
    XOR V9, V0
    SE V1, 0
    LD V7, 1
    RET
column_bit:
    ; V0 = bit for the brick column holding ball x (8 columns of 8 pixels)
    LD V0, V4
    SHR V0
    SHR V0
    SHR V0
    LD I, bit_table
    ADD I, V0
    LD V0, [I]
    RET
draw_row:
    ; V0 = y, V1 = brick mask; draws every set bit as an 8x3 brick
    LD I, brick_sprite
    LD VC, 0
draw_row_next:
    LD VF, V1
    SHR VF
    LD V1, VF
    SE VF, 0
    DRW VC, V0, 3
    ADD VC, 8
    SNE VC, 64
    RET
    JP draw_row_next
reset_ball:
    LD V4, 30
    LD V5, 20
    .patch BALL_DX ball horizontal velocity per frame
    LD V6, 1
    .patch BALL_DY ball vertical velocity per frame
    LD V7, 0xFF
    RET
win:
    JP win
lose:
    JP lose
paddle_sprite:
    DB 0xFF
ball_sprite:
    DB 0x80
brick_sprite:
    DB 0xFE, 0xFE, 0xFE
bit_table:
    DB 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80
